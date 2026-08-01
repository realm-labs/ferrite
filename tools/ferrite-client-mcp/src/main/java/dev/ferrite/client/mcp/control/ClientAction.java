package dev.ferrite.client.mcp.control;

import java.util.Set;

/** Validated action payload accepted by the bounded client-thread queue. */
public sealed interface ClientAction
        permits ClientAction.Look, ClientAction.Inputs, ClientAction.ReleaseAll {
    String actionId();

    String actionName();

    record Look(String actionId, float yaw, float pitch, boolean relative)
            implements ClientAction {
        @Override
        public String actionName() {
            return "look";
        }
    }

    record Inputs(
            String actionId,
            String actionName,
            Set<ControlledInput> inputs,
            boolean down,
            int ticks)
            implements ClientAction {
        public Inputs {
            inputs = Set.copyOf(inputs);
            if (inputs.isEmpty()) {
                throw new IllegalArgumentException("input action must contain at least one input");
            }
            if (down && ticks < 1) {
                throw new IllegalArgumentException("held input duration must be positive");
            }
            if (!down && ticks != 0) {
                throw new IllegalArgumentException("released input duration must be zero");
            }
        }
    }

    record ReleaseAll(String actionId) implements ClientAction {
        @Override
        public String actionName() {
            return "release_all_inputs";
        }
    }
}
