//! `utsim-seguranca`: abre a janela do UTSim Segurança do Trabalho.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use eframe::egui;
use utsim_seguranca_app::{App, TITULO};

fn main() -> ExitCode {
    if let Some(a) = std::env::args().nth(1) {
        match a.as_str() {
            "--versao" | "--version" | "-V" => {
                println!("utsim-seguranca {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--ajuda" | "--help" | "-h" => {
                println!("utsim-seguranca\n  abre a janela do UTSim Segurança do Trabalho");
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITULO)
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([800.0, 520.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        TITULO,
        options,
        Box::new(|cc| {
            utsim_ui_tema::install(&cc.egui_ctx);
            Ok(Box::new(App::new(utsim_posto_ui::sessao::arquivo_padrao())))
        }),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro na janela: {e}");
            ExitCode::FAILURE
        }
    }
}
