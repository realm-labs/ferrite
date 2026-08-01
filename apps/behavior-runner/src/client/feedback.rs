//! Java 26.2 command-result routing and command-block feedback behavior.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColor {
    Gray,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub color: Option<TextColor>,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextComponent {
    Literal(String),
    Translatable {
        key: String,
        arguments: Vec<TextComponent>,
    },
    Sequence(Vec<TextComponent>),
    Styled {
        content: Box<TextComponent>,
        style: TextStyle,
    },
}

impl TextComponent {
    pub fn literal(text: impl Into<String>) -> Self {
        Self::Literal(text.into())
    }

    pub fn translatable(key: impl Into<String>, arguments: Vec<Self>) -> Self {
        Self::Translatable {
            key: key.into(),
            arguments,
        }
    }

    pub fn styled(self, color: TextColor, italic: bool) -> Self {
        Self::Styled {
            content: Box::new(self),
            style: TextStyle {
                color: Some(color),
                italic,
            },
        }
    }

    pub fn plain_text(&self) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Translatable { key, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(Self::plain_text)
                    .collect::<Vec<_>>()
                    .join(", ");
                if arguments.is_empty() {
                    key.clone()
                } else {
                    format!("{key}({arguments})")
                }
            }
            Self::Sequence(parts) => parts.iter().map(Self::plain_text).collect(),
            Self::Styled { content, .. } => content.plain_text(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedbackRules {
    pub command_block_output: bool,
    pub send_command_feedback: bool,
    pub log_admin_commands: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAudience {
    pub id: u64,
    pub display_name: TextComponent,
    pub operator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockFeedback {
    pub track_output: bool,
    pub automatic: bool,
    pub powered: bool,
    pub last_output: Option<TextComponent>,
    pub update_count: u32,
    pub power_update_count: u32,
    closed: bool,
}

impl Default for CommandBlockFeedback {
    fn default() -> Self {
        Self {
            track_output: true,
            automatic: false,
            powered: false,
            last_output: None,
            update_count: 0,
            power_update_count: 0,
            closed: false,
        }
    }
}

impl CommandBlockFeedback {
    pub fn close(&mut self) {
        self.closed = true;
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackSource {
    Player {
        id: u64,
        display_name: TextComponent,
    },
    Server {
        display_name: TextComponent,
        inform_admins: bool,
    },
    Rcon {
        display_name: TextComponent,
        inform_admins: bool,
        buffer: String,
    },
    CommandBlock {
        display_name: TextComponent,
        feedback: CommandBlockFeedback,
    },
    Null,
}

impl FeedbackSource {
    fn accepts_success(&self, rules: FeedbackRules) -> bool {
        match self {
            Self::Player { .. } => rules.send_command_feedback,
            Self::Server { .. } | Self::Rcon { .. } => true,
            Self::CommandBlock { feedback, .. } => {
                feedback.track_output && !feedback.closed && rules.send_command_feedback
            }
            Self::Null => false,
        }
    }

    fn accepts_failure(&self) -> bool {
        match self {
            Self::Player { .. } | Self::Server { .. } | Self::Rcon { .. } => true,
            Self::CommandBlock { feedback, .. } => feedback.track_output && !feedback.closed,
            Self::Null => false,
        }
    }

    fn should_inform_admins(&self, rules: FeedbackRules) -> bool {
        match self {
            Self::Player { .. } => true,
            Self::Server { inform_admins, .. } | Self::Rcon { inform_admins, .. } => *inform_admins,
            Self::CommandBlock { feedback, .. } => {
                feedback.track_output && !feedback.closed && rules.command_block_output
            }
            Self::Null => false,
        }
    }

    fn display_name(&self) -> TextComponent {
        match self {
            Self::Player { display_name, .. }
            | Self::Server { display_name, .. }
            | Self::Rcon { display_name, .. }
            | Self::CommandBlock { display_name, .. } => display_name.clone(),
            Self::Null => TextComponent::literal(""),
        }
    }

    const fn player_id(&self) -> Option<u64> {
        match self {
            Self::Player { id, .. } => Some(*id),
            _ => None,
        }
    }

    const fn is_server(&self) -> bool {
        matches!(self, Self::Server { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackDestination {
    Player(u64),
    ServerLog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackDelivery {
    pub destination: FeedbackDestination,
    pub component: TextComponent,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FeedbackTrace {
    pub deliveries: Vec<FeedbackDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackRouter {
    pub rules: FeedbackRules,
    pub players: Vec<PlayerAudience>,
    pub timestamp: String,
    pub trace: FeedbackTrace,
}

impl FeedbackRouter {
    pub fn send_success<F>(
        &mut self,
        source: &mut FeedbackSource,
        silent: bool,
        broadcast: bool,
        message_supplier: F,
    ) where
        F: FnOnce() -> TextComponent,
    {
        let direct = source.accepts_success(self.rules) && !silent;
        let inform_admins = broadcast && source.should_inform_admins(self.rules) && !silent;
        if !direct && !inform_admins {
            return;
        }

        let message = message_supplier();
        if direct {
            self.deliver_to_source(source, message.clone());
        }
        if inform_admins {
            self.broadcast_to_admins(source, message);
        }
    }

    pub fn send_failure(
        &mut self,
        source: &mut FeedbackSource,
        silent: bool,
        message: TextComponent,
    ) {
        if source.accepts_failure() && !silent {
            self.deliver_to_source(
                source,
                TextComponent::Sequence(vec![TextComponent::literal(""), message])
                    .styled(TextColor::Red, false),
            );
        }
    }

    pub fn route_gamemode_change(
        &mut self,
        source: &mut FeedbackSource,
        silent: bool,
        target_id: u64,
        mode: TextComponent,
        changed: bool,
    ) -> bool {
        if !changed {
            return false;
        }
        if source.player_id() == Some(target_id) {
            self.send_success(source, silent, true, || {
                TextComponent::translatable("commands.gamemode.success.self", vec![mode])
            });
            return true;
        }

        let target_name = self
            .players
            .iter()
            .find(|player| player.id == target_id)
            .map(|player| player.display_name.clone())
            .unwrap_or_else(|| TextComponent::literal(target_id.to_string()));
        if self.rules.send_command_feedback {
            self.trace.deliveries.push(FeedbackDelivery {
                destination: FeedbackDestination::Player(target_id),
                component: TextComponent::translatable("gameMode.changed", vec![mode.clone()]),
            });
        }
        self.send_success(source, silent, true, || {
            TextComponent::translatable("commands.gamemode.success.other", vec![target_name, mode])
        });
        true
    }

    pub fn apply_command_block_placement(
        &self,
        feedback: &mut CommandBlockFeedback,
        block_automatic: bool,
        has_block_entity_data: bool,
        has_neighbor_signal: bool,
    ) {
        if !has_block_entity_data {
            feedback.track_output = self.rules.send_command_feedback;
            feedback.automatic = block_automatic;
        }
        feedback.powered = has_neighbor_signal;
        feedback.power_update_count += 1;
    }

    fn deliver_to_source(&mut self, source: &mut FeedbackSource, component: TextComponent) {
        match source {
            FeedbackSource::Player { id, .. } => self.trace.deliveries.push(FeedbackDelivery {
                destination: FeedbackDestination::Player(*id),
                component,
            }),
            FeedbackSource::Server { .. } => self.trace.deliveries.push(FeedbackDelivery {
                destination: FeedbackDestination::ServerLog,
                component,
            }),
            FeedbackSource::Rcon { buffer, .. } => buffer.push_str(&component.plain_text()),
            FeedbackSource::CommandBlock { feedback, .. } if !feedback.closed => {
                feedback.last_output = Some(TextComponent::Sequence(vec![
                    TextComponent::literal(format!("[{}] ", self.timestamp)),
                    component,
                ]));
                feedback.update_count += 1;
            }
            FeedbackSource::CommandBlock { .. } | FeedbackSource::Null => {}
        }
    }

    fn broadcast_to_admins(&mut self, source: &FeedbackSource, message: TextComponent) {
        let broadcast =
            TextComponent::translatable("chat.type.admin", vec![source.display_name(), message])
                .styled(TextColor::Gray, true);
        if self.rules.send_command_feedback {
            let source_player = source.player_id();
            for player in &self.players {
                if player.operator && source_player != Some(player.id) {
                    self.trace.deliveries.push(FeedbackDelivery {
                        destination: FeedbackDestination::Player(player.id),
                        component: broadcast.clone(),
                    });
                }
            }
        }
        if !source.is_server() && self.rules.log_admin_commands {
            self.trace.deliveries.push(FeedbackDelivery {
                destination: FeedbackDestination::ServerLog,
                component: broadcast,
            });
        }
    }
}
