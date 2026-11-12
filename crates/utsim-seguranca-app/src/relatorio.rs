//! Tela do relatório do inventário: escolhe os postos e grava o HTML em `relatorios/`.

use crate::texto::{fx, tx, vx};
use eframe::egui;
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::inventario::Inventario;
use utsim_seguranca_core::relatorio::{blocos, html};

use crate::AVISO_LEGAL;

#[derive(Default)]
pub struct TelaRelatorio {
    /// Ids dos postos desmarcados (o padrão é todos marcados).
    pub desmarcados: Vec<String>,
}

impl TelaRelatorio {
    /// HTML do inventário dos postos marcados (`marcados` vazio: todos).
    pub fn gerar(s: &Sessao, marcados: &[String]) -> Option<String> {
        let u = s.unidade.as_ref()?;
        let inv = Inventario::ler_com_nos(u);
        let b = blocos(&inv, &u.manifesto.nome, marcados, &util::hoje());
        Some(html(&b))
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        // O aviso legal abre a tela do relatório (e também fica no rodapé).
        ui.colored_label(egui::Color32::from_rgb(230, 160, 40), AVISO_LEGAL);
        let Some(u) = s.unidade.as_ref() else {
            ui.label(tx("relatorio-abra_uma_unidade_na_tela"));
            return;
        };
        ui.heading(tx("relatorio-relatorio_do_inventario_de_riscos"));
        let postos = u.postos();
        if postos.is_empty() {
            ui.label(tx("relatorio-a_unidade_ainda_nao_tem"));
            return;
        }
        ui.label(tx("relatorio-postos_no_relatorio"));
        for p in &postos {
            let mut marcado = !self.desmarcados.contains(&p.id);
            if ui.checkbox(&mut marcado, &p.nome).changed() {
                if marcado {
                    self.desmarcados.retain(|x| x != &p.id);
                } else {
                    self.desmarcados.push(p.id.clone());
                }
            }
        }
        let marcados: Vec<String> = postos
            .iter()
            .filter(|p| !self.desmarcados.contains(&p.id))
            .map(|p| p.id.clone())
            .collect();
        let todos = marcados.len() == postos.len();
        let marcados_pdf = marcados.clone();
        if ui
            .add_enabled(
                !marcados.is_empty(),
                egui::Button::new(tx("relatorio-gravar_html_do_inventario")),
            )
            .clicked()
        {
            let escolha = if todos { Vec::new() } else { marcados };
            if let Some(h) = Self::gerar(s, &escolha)
                && let Some(u) = s.unidade.as_ref()
            {
                match u.gravar_relatorio("inventario_riscos.html", h.as_bytes()) {
                    Ok(p) => s.avisar(fx(
                        "relatorio-relatorio_gravado_em",
                        &[("p", vx(&p.display()))],
                    )),
                    Err(e) => s.falhar(e.to_string()),
                }
            }
        }
        if ui
            .add_enabled(
                !marcados_pdf.is_empty(),
                egui::Button::new(tx("relatorio-gravar_pdf_do_inventario")),
            )
            .clicked()
        {
            let escolha = if todos { Vec::new() } else { marcados_pdf };
            if let Some(u) = s.unidade.as_ref() {
                let inv = Inventario::ler_com_nos(u);
                let hoje = util::hoje();
                let b = blocos(&inv, &u.manifesto.nome, &escolha, &hoje);
                let bytes =
                    utsim_seguranca_core::relatorio::pdf(&b, &format!("{}, {hoje}", u.autor()));
                match u.gravar_relatorio("inventario_riscos.pdf", &bytes) {
                    Ok(p) => s.avisar(fx(
                        "relatorio-relatorio_gravado_em",
                        &[("p", vx(&p.display()))],
                    )),
                    Err(e) => s.falhar(e.to_string()),
                }
            }
        }
    }
}
