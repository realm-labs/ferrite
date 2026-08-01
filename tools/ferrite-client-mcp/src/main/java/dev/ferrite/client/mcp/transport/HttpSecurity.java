package dev.ferrite.client.mcp.transport;

import com.sun.net.httpserver.Headers;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.Locale;

/** Header validation for the local HTTP boundary. */
final class HttpSecurity {
    private static final String BEARER_PREFIX = "Bearer ";

    private HttpSecurity() {}

    static boolean authorized(Headers headers, byte[] expectedSecret) {
        String authorization = headers.getFirst("Authorization");
        if (authorization == null || !authorization.startsWith(BEARER_PREFIX)) {
            return false;
        }
        byte[] supplied = authorization
                .substring(BEARER_PREFIX.length())
                .getBytes(StandardCharsets.UTF_8);
        return MessageDigest.isEqual(expectedSecret, supplied);
    }

    static boolean validOrigin(Headers headers) {
        String origin = headers.getFirst("Origin");
        if (origin == null) {
            return true;
        }
        try {
            URI uri = new URI(origin);
            String host = uri.getHost();
            String scheme = uri.getScheme();
            return scheme != null
                    && (scheme.equalsIgnoreCase("http") || scheme.equalsIgnoreCase("https"))
                    && uri.getUserInfo() == null
                    && (uri.getPath() == null || uri.getPath().isEmpty())
                    && uri.getQuery() == null
                    && uri.getFragment() == null
                    && host != null
                    && (host.equalsIgnoreCase("localhost")
                            || host.equals("127.0.0.1")
                            || host.equals("::1"));
        } catch (URISyntaxException error) {
            return false;
        }
    }

    static boolean acceptsMcpResponse(Headers headers) {
        String accept = headers.getFirst("Accept");
        if (accept == null) {
            return false;
        }
        String normalized = accept.toLowerCase(Locale.ROOT);
        return normalized.contains("application/json")
                && normalized.contains("text/event-stream");
    }

    static boolean isJsonRequest(Headers headers) {
        String contentType = headers.getFirst("Content-Type");
        return contentType != null
                && contentType
                        .toLowerCase(Locale.ROOT)
                        .split(";", 2)[0]
                        .strip()
                        .equals("application/json");
    }
}
