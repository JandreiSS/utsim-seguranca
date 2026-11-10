//! `utsim-seguranca`: abre a janela do UTSim Segurança do Trabalho.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use eframe::egui;
use utsim_seguranca_app::{App, ICONE, TITULO};

fn main() -> ExitCode {
    if let Some(a) = std::env::args().nth(1) {
        match a.as_str() {
            "--versao" | "--version" | "-V" => {
                println!("utsim-seguranca {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--ajuda" | "--help" | "-h" => {
                println!(
                    "utsim-seguranca\n  abre a janela do UTSim Segurança do Trabalho
  --utsim-abrir=<modelo.ron> --utsim-no=<nome>   abre o modelo no nó
  --utsim-abrir=<dados.csv|.xlsx> --utsim-ferramenta=<chave>   abre os dados numa ferramenta"
                );
                return ExitCode::SUCCESS;
            }
            _ => {}
        }
    }
    // `--utsim-abrir=<modelo.ron> --utsim-no=<nome>` ou `--utsim-abrir=<dados.csv>`.
    let pedido = utsim_posto_ui::Abrir::de_args(std::env::args().skip(1));
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(TITULO)
        .with_inner_size([1180.0, 780.0])
        .with_min_inner_size([800.0, 520.0])
        .with_maximized(true);
    if let Some((w, h, rgba)) = utsim_ui_tema::familia::png_rgba(ICONE) {
        viewport = viewport.with_icon(std::sync::Arc::new(egui::IconData {
            rgba,
            width: w as u32,
            height: h as u32,
        }));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let result = eframe::run_native(
        TITULO,
        options,
        Box::new(|cc| {
            utsim_ui_tema::install(&cc.egui_ctx);
            utsim_ui_tema::usar_cor_do_app(&cc.egui_ctx, utsim_seguranca_app::app::MARCA.cor);
            let mut app = App::new(utsim_posto_ui::sessao::arquivo_padrao());
            if !pedido.vazio() {
                app.atender(&pedido);
            }
            Ok(Box::new(app))
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
