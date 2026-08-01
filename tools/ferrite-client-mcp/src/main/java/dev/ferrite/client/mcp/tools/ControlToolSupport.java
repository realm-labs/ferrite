package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ActionReceipt;
import dev.ferrite.client.mcp.control.ActionState;

/** Strict JSON parsing and receipt formatting shared by control tools. */
final class ControlToolSupport {
    static final int MAXIMUM_HELD_TICKS = 200;

    private ControlToolSupport() {}

    static String actionId(JsonObject arguments) {
        return string(arguments, "actionId");
    }

    static String string(JsonObject arguments, String name) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isString()) {
            throw new IllegalArgumentException(name + " must be a string");
        }
        return value.getAsString();
    }

    static boolean bool(JsonObject arguments, String name) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isBoolean()) {
            throw new IllegalArgumentException(name + " must be a boolean");
        }
        return value.getAsBoolean();
    }

    static boolean optionalBool(JsonObject arguments, String name, boolean fallback) {
        return arguments.has(name) ? bool(arguments, name) : fallback;
    }

    static int boundedInt(JsonObject arguments, String name, int minimum, int maximum) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isNumber()) {
            throw new IllegalArgumentException(name + " must be an integer");
        }
        double number = value.getAsDouble();
        int integer = value.getAsInt();
        if (!Double.isFinite(number) || number != integer) {
            throw new IllegalArgumentException(name + " must be an integer");
        }
        if (integer < minimum || integer > maximum) {
            throw new IllegalArgumentException(
                    name + " must be between " + minimum + " and " + maximum);
        }
        return integer;
    }

    static long nonNegativeLong(JsonObject arguments, String name) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isNumber()) {
            throw new IllegalArgumentException(name + " must be an integer");
        }
        double number = value.getAsDouble();
        long integer = value.getAsLong();
        if (!Double.isFinite(number) || number != integer || integer < 0) {
            throw new IllegalArgumentException(name + " must be a non-negative integer");
        }
        return integer;
    }

    static float finiteFloat(JsonObject arguments, String name, float minimum, float maximum) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isNumber()) {
            throw new IllegalArgumentException(name + " must be a number");
        }
        float number = value.getAsFloat();
        if (!Float.isFinite(number) || number < minimum || number > maximum) {
            throw new IllegalArgumentException(
                    name + " must be finite and between " + minimum + " and " + maximum);
        }
        return number;
    }

    static double finiteDouble(JsonObject arguments, String name, double minimum, double maximum) {
        JsonElement value = arguments.get(name);
        if (value == null
                || !value.isJsonPrimitive()
                || !value.getAsJsonPrimitive().isNumber()) {
            throw new IllegalArgumentException(name + " must be a number");
        }
        double number = value.getAsDouble();
        if (!Double.isFinite(number) || number < minimum || number > maximum) {
            throw new IllegalArgumentException(
                    name + " must be finite and between " + minimum + " and " + maximum);
        }
        return number;
    }

    static McpToolResult receipt(ActionReceipt receipt) {
        JsonObject json = receiptJson(receipt);
        boolean error = receipt.state() == ActionState.REJECTED
                || receipt.state() == ActionState.CANCELLED
                || receipt.state() == ActionState.TIMED_OUT;
        return new McpToolResult(
                json,
                receipt.action() + " is " + receipt.state().wireName() + ": " + receipt.detail(),
                error);
    }

    static JsonObject receiptJson(ActionReceipt receipt) {
        JsonObject json = new JsonObject();
        json.addProperty("actionId", receipt.actionId());
        json.addProperty("action", receipt.action());
        json.addProperty("state", receipt.state().wireName());
        json.addProperty("acceptedTick", receipt.acceptedTick());
        if (receipt.appliedTick() != null) {
            json.addProperty("appliedTick", receipt.appliedTick());
        }
        if (receipt.completedTick() != null) {
            json.addProperty("completedTick", receipt.completedTick());
        }
        json.addProperty("detail", receipt.detail());
        return json;
    }
}
