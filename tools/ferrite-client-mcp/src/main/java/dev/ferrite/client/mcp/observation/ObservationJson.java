package dev.ferrite.client.mcp.observation;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;

/** Stable record-to-JSON adapter for immutable observations. */
public final class ObservationJson {
    private static final Gson GSON = new GsonBuilder().serializeNulls().create();

    private ObservationJson() {}

    public static JsonObject object(Object value) {
        return GSON.toJsonTree(value).getAsJsonObject();
    }
}
