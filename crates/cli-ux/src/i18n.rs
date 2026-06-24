//! Human-facing CLI strings keyed by [`project_config::AgentLanguage`].

use crate::project_config::AgentLanguage;

pub fn global_flags(lang: AgentLanguage) -> &'static str {
    match lang {
        AgentLanguage::ZhCn => {
            "--format json | --project <path>（任意命令）；环境变量 POPSICLE_PROJECT"
        }
        AgentLanguage::En => "--format json | --project <path> (any command); env POPSICLE_PROJECT",
    }
}

pub fn help_next(lang: AgentLanguage) -> &'static str {
    match lang {
        AgentLanguage::ZhCn => "popsicle doctor --format json",
        AgentLanguage::En => "popsicle doctor --format json",
    }
}

pub fn issue_create_next(lang: AgentLanguage, key: &str) -> String {
    match lang {
        AgentLanguage::ZhCn => format!("popsicle issue show {key} --format json"),
        AgentLanguage::En => format!("popsicle issue start {key}"),
    }
}

pub fn issue_show_next(lang: AgentLanguage, key: &str) -> String {
    match lang {
        AgentLanguage::ZhCn => format!("popsicle issue start {key}"),
        AgentLanguage::En => format!("popsicle issue start {key}"),
    }
}

pub fn issue_list_next(lang: AgentLanguage) -> &'static str {
    match lang {
        AgentLanguage::ZhCn => "popsicle issue show <key> --format json",
        AgentLanguage::En => "popsicle issue show <key>",
    }
}

pub fn init_issue_create_hint(lang: AgentLanguage) -> &'static str {
    match lang {
        AgentLanguage::ZhCn => {
            "popsicle issue create --type <product|technical|bug|idea> --title \"<简体中文标题>\" --product <产品> [--pipeline <名称>]"
        }
        AgentLanguage::En => {
            "popsicle issue create --type <product|technical|bug|idea> --title \"<title>\" --product <id> [--pipeline <name>]"
        }
    }
}

pub fn command_usage(lang: AgentLanguage) -> &'static [&'static str] {
    match lang {
        AgentLanguage::ZhCn => COMMAND_USAGE_ZH,
        AgentLanguage::En => COMMAND_USAGE_EN,
    }
}

const COMMAND_USAGE_EN: &[&str] = &[
    "popsicle init",
    "popsicle doctor [--format json]",
    "popsicle issue create --type <product|technical|bug|idea> --title \"<t>\" --product <id> [--pipeline <name>]",
    "popsicle issue list",
    "popsicle issue show <key>",
    "popsicle issue start <key> [--product <id>] [--pipeline <name>]",
    "popsicle issue close <key>",
    "popsicle issue link <key> --tasks T1,T2 [--replace] [--drop-proposed]",
    "popsicle pipeline status --run <run_id>",
    "popsicle pipeline next --run <run_id>",
    "popsicle pipeline stage complete <stage> --run <run_id> [--confirm]",
    "popsicle doc create <skill> --title \"<t>\" --run <run_id>",
    "popsicle doc list [--run <run_id>]",
    "popsicle doc show <doc_id>",
    "popsicle doc check <doc_id>",
    "popsicle tool run <tool> key=value ...  # intent-validate path=products; mermaid-diagram action=guide",
    "popsicle admin migrate [--workspace <path>]",
    "popsicle admin reinit [--workspace <path>]",
    "popsicle admin sync-project-config [--workspace <path>]",
    "popsicle admin backfill-pipeline-names [--dry-run] [--workspace <path>]",
    "popsicle admin purge-legacy-workspace [--dry-run] [--workspace <path>]",
    "popsicle admin relocate-workspace [--dry-run] [--workspace <path>]",
    "popsicle project list",
    "popsicle project add <path> [--name <name>]",
    "popsicle project use <name|path>",
    "popsicle project remove <name>",
    "popsicle project current",
    "popsicle ui [--project <path>]",
];

const COMMAND_USAGE_ZH: &[&str] = &[
    "popsicle init",
    "popsicle doctor [--format json]",
    "popsicle issue create --type <product|technical|bug|idea> --title \"<标题>\" --product <产品> [--pipeline <名称>]",
    "popsicle issue list",
    "popsicle issue show <key>",
    "popsicle issue start <key> [--product <产品>] [--pipeline <名称>]",
    "popsicle issue close <key>",
    "popsicle issue link <key> --tasks T1,T2 [--replace] [--drop-proposed]",
    "popsicle pipeline status --run <run_id>",
    "popsicle pipeline next --run <run_id>",
    "popsicle pipeline stage complete <stage> --run <run_id> [--confirm]",
    "popsicle doc create <skill> --title \"<标题>\" --run <run_id>",
    "popsicle doc list [--run <run_id>]",
    "popsicle doc show <doc_id>",
    "popsicle doc check <doc_id>",
    "popsicle tool run <tool> key=value ...  # intent-validate path=products; mermaid-diagram action=guide",
    "popsicle admin migrate [--workspace <path>]",
    "popsicle admin reinit [--workspace <path>]",
    "popsicle admin sync-project-config [--workspace <path>]",
    "popsicle admin backfill-pipeline-names [--dry-run] [--workspace <path>]",
    "popsicle admin purge-legacy-workspace [--dry-run] [--workspace <path>]",
    "popsicle admin relocate-workspace [--dry-run] [--workspace <path>]",
    "popsicle project list",
    "popsicle project add <path> [--name <名称>]",
    "popsicle project use <名称|路径>",
    "popsicle project remove <名称>",
    "popsicle project current",
    "popsicle ui [--project <path>]",
];
