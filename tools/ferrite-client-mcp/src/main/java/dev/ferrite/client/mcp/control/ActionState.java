package dev.ferrite.client.mcp.control;

/** Stable lifecycle states exposed by mutating MCP tools. */
public enum ActionState {
    QUEUED("Queued"),
    APPLIED("Applied"),
    SATISFIED("Satisfied"),
    TIMED_OUT("TimedOut"),
    CANCELLED("Cancelled"),
    REJECTED("Rejected");

    private final String wireName;

    ActionState(String wireName) {
        this.wireName = wireName;
    }

    public String wireName() {
        return wireName;
    }

    public boolean completed() {
        return this != QUEUED && this != APPLIED;
    }
}
