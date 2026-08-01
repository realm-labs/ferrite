package dev.ferrite.client.mcp.acceptance;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.BufferedReader;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;

final class ProcessTreeTest {
    @Test
    void terminatesTheOwnedParentAndDescendant() throws Exception {
        Process parent = fixture("parent");
        long childPid;
        try (BufferedReader output = parent.inputReader(StandardCharsets.UTF_8)) {
            childPid = Long.parseLong(output.readLine());
        }
        ProcessHandle child = ProcessHandle.of(childPid).orElseThrow();
        assertTrue(parent.isAlive());
        assertTrue(child.isAlive());

        ProcessTree.terminate(parent, Duration.ofSeconds(1));

        assertFalse(parent.isAlive());
        child.onExit().get(5, TimeUnit.SECONDS);
        assertFalse(child.isAlive());
    }

    @Test
    void terminatingAnAlreadyCrashedProcessIsSafe() throws Exception {
        Process crashed = fixture("crash");
        assertTrue(crashed.waitFor(5, TimeUnit.SECONDS));
        assertEquals(17, crashed.exitValue());

        ProcessTree.terminate(crashed, Duration.ofMillis(100));
        assertFalse(crashed.isAlive());
    }

    private static Process fixture(String mode) throws Exception {
        String java = ProcessHandle.current().info().command().orElseThrow();
        return new ProcessBuilder(
                        java,
                        "-cp",
                        System.getProperty("java.class.path"),
                        ProcessFixture.class.getName(),
                        mode)
                .redirectErrorStream(true)
                .start();
    }

    public static final class ProcessFixture {
        private ProcessFixture() {}

        public static void main(String[] arguments) throws Exception {
            if (arguments.length == 1 && "crash".equals(arguments[0])) {
                System.exit(17);
            }
            if (arguments.length == 1 && "parent".equals(arguments[0])) {
                Process child = fixture("sleep");
                System.out.println(child.pid());
                System.out.flush();
                Thread.sleep(Duration.ofMinutes(2));
                return;
            }
            if (arguments.length == 1 && "sleep".equals(arguments[0])) {
                Thread.sleep(Duration.ofMinutes(2));
                return;
            }
            System.exit(2);
        }
    }
}
