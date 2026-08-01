package dev.ferrite.client.mcp.acceptance;

import java.util.ArrayList;
import java.util.List;

/** Pure-Java entry point for unattended reference and Ferrite gameplay evidence. */
public final class AcceptanceRunner {
    private AcceptanceRunner() {}

    public static void main(String[] arguments) {
        try {
            AcceptanceConfig config = AcceptanceConfig.parse(arguments);
            List<String> failures = new ArrayList<>();
            if (config.mode() == AcceptanceConfig.Mode.REFERENCE
                    || config.mode() == AcceptanceConfig.Mode.ALL) {
                run(config, "reference-gameplay", failures, GameplayScenario::runReference);
            }
            if (config.mode() == AcceptanceConfig.Mode.FERRITE
                    || config.mode() == AcceptanceConfig.Mode.ALL) {
                run(config, "ferrite-visual", failures, GameplayScenario::runFerrite);
            }
            if (config.mode() == AcceptanceConfig.Mode.FERRITE_PORTAL
                    || config.mode() == AcceptanceConfig.Mode.ALL) {
                run(config, "ferrite-portal", failures, GameplayScenario::runFerritePortal);
            }
            if (!failures.isEmpty()) {
                System.err.println("acceptance failed: " + String.join(", ", failures));
                System.exit(1);
            }
            System.out.println("acceptance satisfied");
        } catch (IllegalArgumentException error) {
            System.err.println("acceptance rejected: invalid arguments");
            System.exit(2);
        } catch (Exception error) {
            System.err.println("acceptance failed before scenario execution: " + error.getMessage());
            System.exit(1);
        }
    }

    private static void run(
            AcceptanceConfig config,
            String name,
            List<String> failures,
            Scenario scenario)
            throws Exception {
        EvidenceBundle evidence = EvidenceBundle.create(config.outputRoot(), name);
        try {
            scenario.run(config, evidence);
            evidence.finish("Satisfied", "all deterministic scenario assertions passed");
            System.out.println(name + " satisfied: " + evidence.root());
        } catch (Exception error) {
            evidence.finish("Failed", error.getClass().getSimpleName() + ": " + error.getMessage());
            failures.add(name + " (" + evidence.root() + ")");
        }
    }

    @FunctionalInterface
    private interface Scenario {
        void run(AcceptanceConfig config, EvidenceBundle evidence) throws Exception;
    }
}
