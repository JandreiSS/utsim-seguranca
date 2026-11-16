//! `utsim-seguranca`: abre a janela do UTSim Segurança do Trabalho.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use utsim_seguranca_app::{App, ICONE, TITULO};

fn main() -> ExitCode {
    // A primeira vez segue a língua do sistema; a guardada vale por cima (`App::new`).
    utsim_idiomas::definir(utsim_idiomas::Idioma::inicial());
    if let Some(c) = utsim_ui_tema::janela::versao_ou_ajuda(
        std::env::args().nth(1).as_deref(),
        "utsim-seguranca",
        env!("CARGO_PKG_VERSION"),
        "utsim-seguranca\n  abre a janela do UTSim Segurança do Trabalho
  --utsim-abrir=<modelo.ron> --utsim-no=<nome>   abre o modelo no nó
  --utsim-abrir=<dados.csv|.xlsx> --utsim-ferramenta=<chave>   abre os dados numa ferramenta
  --utsim-pedido=<id.utsim-pedido.json>   atende o pedido de outro app e responde ao fechar",
    ) {
        return c;
    }
    // `--utsim-abrir=<modelo.ron> --utsim-no=<nome>` ou `--utsim-abrir=<dados.csv>`.
    let pedido = utsim_posto_ui::Abrir::de_args(std::env::args().skip(1));
    let options = eframe::NativeOptions {
        viewport: utsim_ui_tema::janela::viewport(
            TITULO,
            [1180.0, 780.0],
            [800.0, 520.0],
            Some(ICONE),
        ),
        ..Default::default()
    };
    let result = eframe::run_native(
        TITULO,
        options,
        Box::new(|cc| {
            utsim_ui_tema::install(&cc.egui_ctx);
            utsim_ui_tema::usar_cor_do_app(&cc.egui_ctx, utsim_seguranca_app::app::COR_APP);
            let mut app = App::new(utsim_posto_ui::sessao::arquivo_padrao());
            if !pedido.vazio() {
                app.atender(&pedido);
            }
            Ok(Box::new(app))
        }),
    );
    utsim_ui_tema::janela::saida(result)
}
