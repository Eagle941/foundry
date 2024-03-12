//! Debugger implementation.

use alloy_primitives::Address;
use eyre::Result;
use foundry_common::{compile::ContractSources, evm::Breakpoints};
use foundry_evm_core::{debug::DebugNodeFlat, utils::PcIcMap};
use revm::primitives::SpecId;
use std::{collections::HashMap, path::PathBuf};

use crate::{context::DebuggerContext, tui::TUI, DebuggerBuilder, ExitReason, FileDumper};

pub struct Debugger {
    context: DebuggerContext,
}

impl Debugger {
    /// Creates a new debugger builder.
    #[inline]
    pub fn builder() -> DebuggerBuilder {
        DebuggerBuilder::new()
    }

    /// Creates a new debugger.
    pub fn new(
        debug_arena: Vec<DebugNodeFlat>,
        identified_contracts: HashMap<Address, String>,
        contracts_sources: ContractSources,
        breakpoints: Breakpoints,
    ) -> Self {
        let pc_ic_maps = contracts_sources
            .entries()
            .filter_map(|(contract_name, (_, contract, _))| {
                Some((
                    contract_name.clone(),
                    (
                        PcIcMap::new(SpecId::LATEST, contract.bytecode.bytes()?),
                        PcIcMap::new(SpecId::LATEST, contract.deployed_bytecode.bytes()?),
                    ),
                ))
            })
            .collect();
        Self {
            context: DebuggerContext {
                debug_arena,
                identified_contracts,
                contracts_sources,
                pc_ic_maps,
                breakpoints,
            },
        }
    }

    /// Starts the debugger TUI. Terminates the current process on failure or user exit.
    pub fn run_tui_exit(mut self) -> ! {
        let code = match self.try_run_tui() {
            Ok(ExitReason::CharExit) => 0,
            Err(e) => {
                println!("{e}");
                1
            }
        };
        std::process::exit(code)
    }

    /// Starts the debugger TUI.
    pub fn try_run_tui(&mut self) -> Result<ExitReason> {
        eyre::ensure!(!self.context.debug_arena.is_empty(), "debug arena is empty");

        let mut tui = TUI::new(&mut self.context);
        tui.try_run()
    }

    /// Dumps debugger data to file.
    pub fn dump_to_file(&mut self, path: &PathBuf) -> Result<()> {
        eyre::ensure!(!self.context.debug_arena.is_empty(), "debug arena is empty");

        let mut file_dumper = FileDumper::new(path, &mut self.context);
        file_dumper.run()
    }
}
