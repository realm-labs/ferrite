use crate::verification::{documented_rule_ids, load_experiments};
use crate::*;

pub(crate) fn experiments(context: &Context, command: ExperimentCommand) -> Result<()> {
    let definitions = load_experiments(context)?;
    match command {
        ExperimentCommand::List => {
            for experiment in definitions {
                println!(
                    "{}\t{}\t{}",
                    experiment.id, experiment.mode, experiment.status
                );
            }
            Ok(())
        }
        ExperimentCommand::Verify => validate_experiments(context, &definitions),
        ExperimentCommand::Run { id } => {
            let experiment = definitions
                .iter()
                .find(|experiment| experiment.id == id)
                .with_context(|| format!("unknown experiment {id}"))?;
            let run_directory = context.cache.join("experiments").join(&id);
            fs::create_dir_all(&run_directory)?;
            fs::write(
                run_directory.join("procedure.json"),
                serde_json::to_vec_pretty(experiment)?,
            )?;
            ensure!(
                experiment.status == "automated",
                "{} is {}; prepared {} but no automated result can be claimed",
                id,
                experiment.status,
                run_directory.join("procedure.json").display()
            );
            let runner = context
                .reference
                .join("experiments/runner")
                .join(format!("{id}.sh"));
            ensure!(
                runner.is_file(),
                "automated runner is not committed for {id}"
            );
            let result_path = run_directory.join("result.json");
            if result_path.exists() {
                fs::remove_file(&result_path)?;
            }
            let status = ProcessCommand::new("sh")
                .arg(runner)
                .current_dir(&run_directory)
                .env("MC_REF_CACHE", &context.cache)
                .env("MC_REF_EXPERIMENT_DIR", &run_directory)
                .env("MC_REF_SERVER_JAR", context.cache.join("server.jar"))
                .status()?;
            ensure!(status.success(), "experiment {id} failed");
            let result: ExperimentResult = serde_json::from_reader(BufReader::new(
                File::open(&result_path)
                    .with_context(|| format!("runner did not produce {}", result_path.display()))?,
            ))?;
            ensure!(
                result.passed && !result.observations.is_empty(),
                "experiment {id} did not pass with recorded observations"
            );
            println!(
                "experiment {id} passed with {} observations",
                result.observations.len()
            );
            Ok(())
        }
    }
}

fn validate_experiments(context: &Context, definitions: &[Experiment]) -> Result<()> {
    let rules = documented_rule_ids(context)?;
    let mut ids = BTreeSet::new();
    for experiment in definitions {
        ensure!(
            experiment.id.starts_with("EXP-"),
            "invalid experiment ID {}",
            experiment.id
        );
        ensure!(
            ids.insert(&experiment.id),
            "duplicate experiment ID {}",
            experiment.id
        );
        ensure!(
            experiment.repeats > 0,
            "{} repeats must be positive",
            experiment.id
        );
        ensure!(
            !experiment.initial_state.is_empty()
                && !experiment.action.is_empty()
                && !experiment.observation.is_empty()
                && !experiment.expected.is_empty(),
            "{} has an incomplete procedure",
            experiment.id
        );
        ensure!(
            experiment
                .action
                .windows(2)
                .all(|pair| pair[0].tick <= pair[1].tick),
            "{} actions are not tick ordered",
            experiment.id
        );
        ensure!(
            experiment
                .observation
                .windows(2)
                .all(|pair| pair[0].tick <= pair[1].tick),
            "{} observations are not tick ordered",
            experiment.id
        );
        ensure!(
            experiment.action.iter().all(|v| !v.value.trim().is_empty())
                && experiment
                    .observation
                    .iter()
                    .all(|v| !v.value.trim().is_empty()),
            "{} contains an empty step",
            experiment.id
        );
        for rule in &experiment.rules {
            ensure!(
                rules.contains(rule),
                "{} references missing rule {rule}",
                experiment.id
            );
        }
    }
    println!("experiment definitions verified: {}", definitions.len());
    Ok(())
}
