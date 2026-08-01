package dev.ferrite.client.mcp.observation;

import java.util.regex.Pattern;

/** Conservative redaction for client messages exposed through MCP evidence. */
public final class SensitiveText {
    private static final int MAXIMUM_MESSAGE_LENGTH = 512;
    private static final Pattern CREDENTIAL = Pattern.compile(
            "(?i)(access[_ -]?token|client[_ -]?token|authorization|bearer)\\s*[:=]?\\s*\\S+");

    private SensitiveText() {}

    public static String redact(String message) {
        String redacted = message == null ? "unknown client error" : message;
        String userHome = System.getProperty("user.home");
        if (userHome != null && !userHome.isBlank()) {
            redacted = redacted.replace(userHome, "<user-home>");
        }
        redacted = CREDENTIAL.matcher(redacted).replaceAll("$1=<redacted>");
        if (redacted.length() > MAXIMUM_MESSAGE_LENGTH) {
            return redacted.substring(0, MAXIMUM_MESSAGE_LENGTH) + "…";
        }
        return redacted;
    }
}
