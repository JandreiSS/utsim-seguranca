//! Tela de exposição simulada: tempo em zona, entradas sem EPI e quase-conflitos.

use crate::texto::{fx, tx, vx};
use eframe::egui;
use utsim_exposicao::relatorio as rel;
use utsim_exposicao::{Exposure, Options, analyze_all, interval};
use utsim_model::Model;
use utsim_posto_ui::{Sessao, Trabalho, util};
use utsim_seguranca_core::inventario::{PERIGOS, Perigo, perigos_da_exposicao};

pub struct Resultado {
    pub modelo: Model,
    pub replicacoes: Vec<Exposure>,
}

#[derive(Default)]
pub struct TelaExposicao {
    pub modelo: String,
    pub trabalho: Trabalho<Resultado>,
    pub sugestoes: Vec<Perigo>,
    pub escolhidos: Vec<bool>,
}

fn media(ui: &mut egui::Ui, nome: &str, valores: &[f64]) {
    let i = interval(valores);
    ui.label(nome);
    ui.label(utsim_idiomas::marca_decimal(&format!(
        "{:.2} ± {:.2}",
        i.mean, i.half_width
    )));
    ui.end_row();
}

/// Pessoas sob a carga suspensa das pontes rolantes (primeira replicação). A meta é zero:
/// qualquer tempo sob a carga aparece em vermelho.
fn sob_carga_ui(ui: &mut egui::Ui, reps: &[Exposure]) {
    let vermelho = egui::Color32::from_rgb(230, 90, 80);
    let total: f64 = reps.iter().map(Exposure::under_load_time).sum();
    ui.add_space(4.0);
    ui.strong(tx("exposicao-sob_carga_suspensa_primeira"));
    if total > 0.0 {
        ui.colored_label(vermelho, tx("exposicao-alguem_ficou_sob_a_carga"));
    } else {
        ui.label(tx("exposicao-ninguem_sob_a_carga"));
    }
    let Some(e0) = reps.first() else {
        return;
    };
    egui::Grid::new("exp_sob_carga")
        .striped(true)
        .show(ui, |ui| {
            for c in [
                tx("exposicao-ponte"),
                tx("exposicao-pessoa"),
                tx("exposicao-tempo_s"),
                tx("exposicao-entradas"),
            ] {
                ui.strong(c);
            }
            ui.end_row();
            for c in &e0.under_load {
                ui.label(&c.crane);
                ui.label(&c.person);
                let t = utsim_idiomas::marca_decimal(&format!("{:.1}", c.time));
                if c.time > 0.0 {
                    ui.colored_label(vermelho, t);
                } else {
                    ui.label(t);
                }
                ui.label(c.entries.to_string());
                ui.end_row();
            }
        });
}

impl TelaExposicao {
    pub fn rodar(&mut self, s: &mut Sessao) {
        let Some(u) = s.unidade.as_ref() else {
            return;
        };
        let modelo = match util::carregar_modelo(u, &self.modelo) {
            Ok(m) => m,
            Err(e) => {
                s.falhar(e);
                return;
            }
        };
        self.sugestoes.clear();
        self.escolhidos.clear();
        s.avisar(tx("exposicao-calculando_a_exposicao"));
        self.trabalho.iniciar(move || {
            let replicacoes =
                analyze_all(&modelo, Options::default()).map_err(|e| e.to_string())?;
            Ok(Resultado {
                modelo,
                replicacoes,
            })
        });
    }

    /// Perigos sugeridos pela exposição, sem repetir os que a unidade já tem.
    pub fn sugerir(&mut self, s: &Sessao) {
        let (Some(u), Some(r)) = (s.unidade.as_ref(), self.trabalho.ok()) else {
            return;
        };
        let Some(e0) = r.replicacoes.first() else {
            return;
        };
        let existentes: Vec<Perigo> = u.estado().lista(PERIGOS);
        self.sugestoes = perigos_da_exposicao(&u.postos(), e0, &r.modelo)
            .into_iter()
            .filter(|n| {
                !existentes
                    .iter()
                    .any(|p| p.posto == n.posto && p.descricao == n.descricao)
            })
            .collect();
        self.escolhidos = vec![true; self.sugestoes.len()];
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        if s.unidade.is_none() {
            ui.label(tx("exposicao-abra_uma_unidade_na_tela"));
            return;
        }
        if self.trabalho.sondar(ui.ctx()) {
            match self.trabalho.erro() {
                Some(e) => s.falhar(e.to_string()),
                None => s.avisar(tx("exposicao-exposicao_calculada")),
            }
        }
        ui.heading(tx("exposicao-exposicao_simulada"));
        ui.colored_label(egui::Color32::from_rgb(230, 160, 40), rel::AVISO_SIMULADO);
        let modelos: Vec<String> = {
            let mut v: Vec<String> = s
                .unidade
                .as_ref()
                .map(|u| u.postos())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| p.modelo)
                .collect();
            v.sort();
            v.dedup();
            v
        };
        ui.horizontal(|ui| {
            ui.label(tx("exposicao-modelo_ron"));
            utsim_posto_ui::util::campo_modelo(ui, &mut self.modelo, s.unidade.as_ref(), 420.0);
            if !modelos.is_empty() {
                egui::ComboBox::from_id_salt("modelos_exposicao")
                    .selected_text(tx("exposicao-modelos_dos_postos"))
                    .show_ui(ui, |ui| {
                        for m in &modelos {
                            ui.selectable_value(&mut self.modelo, m.clone(), m);
                        }
                    });
            }
            let livre = !self.trabalho.ocupado() && !self.modelo.trim().is_empty();
            if ui
                .add_enabled(livre, egui::Button::new(tx("exposicao-rodar")))
                .clicked()
            {
                self.rodar(s);
            }
            if self.trabalho.ocupado() {
                ui.spinner();
            }
        });
        let Some(r) = self.trabalho.ok() else {
            if let Some(e) = self.trabalho.erro() {
                ui.colored_label(egui::Color32::from_rgb(230, 90, 80), e);
            }
            return;
        };
        ui.separator();
        ui.strong(fx(
            "exposicao-replicacoes_origem_simulado",
            &[("replicacoes", vx(&r.replicacoes.len()))],
        ));
        let tempo: Vec<f64> = r
            .replicacoes
            .iter()
            .map(|e| e.zones.iter().map(|z| z.time).sum())
            .collect();
        let sem_epi: Vec<f64> = r
            .replicacoes
            .iter()
            .map(|e| f64::from(e.entries_without_ppe()))
            .collect();
        let quase: Vec<f64> = r
            .replicacoes
            .iter()
            .map(|e| e.near_miss_count() as f64)
            .collect();
        let tem_ponte = r.replicacoes.iter().any(|e| !e.under_load.is_empty());
        let sob_carga: Vec<f64> = r
            .replicacoes
            .iter()
            .map(Exposure::under_load_time)
            .collect();
        let entradas_carga: Vec<f64> = r
            .replicacoes
            .iter()
            .map(|e| f64::from(e.under_load_entries()))
            .collect();
        egui::Grid::new("exp_resumo").striped(true).show(ui, |ui| {
            ui.strong(tx("exposicao-indicador"));
            ui.strong(tx("exposicao-media_ic_95"));
            ui.end_row();
            media(ui, tx("exposicao-tempo_das_pessoas_em_zona"), &tempo);
            media(ui, tx("exposicao-entradas_em_zona_sem_o"), &sem_epi);
            media(ui, tx("exposicao-quase_conflitos_simulados"), &quase);
            if tem_ponte {
                media(ui, tx("exposicao-tempo_sob_carga_suspensa"), &sob_carga);
                media(
                    ui,
                    tx("exposicao-entradas_sob_carga_suspensa"),
                    &entradas_carga,
                );
            }
        });
        if tem_ponte {
            sob_carga_ui(ui, &r.replicacoes);
        }
        if let Some(e0) = r.replicacoes.first() {
            ui.add_space(4.0);
            ui.strong(tx("exposicao-por_zona_e_pessoa_primeira"));
            egui::Grid::new("exp_zonas").striped(true).show(ui, |ui| {
                for c in [
                    tx("exposicao-zona"),
                    tx("exposicao-pessoa"),
                    tx("exposicao-tempo_s"),
                    tx("exposicao-entradas"),
                    tx("exposicao-sem_epi"),
                    tx("exposicao-epi_faltante"),
                ] {
                    ui.strong(c);
                }
                ui.end_row();
                for z in &e0.zones {
                    ui.label(&z.zone);
                    ui.label(&z.person);
                    ui.label(utsim_idiomas::marca_decimal(&format!("{:.1}", z.time)));
                    ui.label(z.entries.to_string());
                    ui.label(z.entries_without_ppe.to_string());
                    ui.label(z.missing_ppe.join(", "));
                    ui.end_row();
                }
            });
        }
        ui.add_space(6.0);
        let mut sugerir = false;
        let mut gravar_html = false;
        ui.horizontal(|ui| {
            sugerir = ui.button(tx("exposicao-sugerir_perigos")).clicked();
            gravar_html = ui
                .button(tx("exposicao-gravar_html_de_exposicao"))
                .clicked();
        });
        if gravar_html {
            let html = rel::html(&r.modelo, &r.replicacoes);
            if let Some(u) = s.unidade.as_ref() {
                match u.gravar_relatorio("exposicao.html", html.as_bytes()) {
                    Ok(p) => s.avisar(fx(
                        "exposicao-relatorio_gravado_em",
                        &[("p", vx(&p.display()))],
                    )),
                    Err(e) => s.falhar(e.to_string()),
                }
            }
        }
        if sugerir {
            self.sugerir(s);
            let n = self.sugestoes.len();
            s.avisar(fx(
                "exposicao-perigos_sugeridos_nenhum_gravado_ainda",
                &[("n", vx(&n))],
            ));
        }
        if !self.sugestoes.is_empty() {
            let nomes: Vec<String> = s
                .unidade
                .as_ref()
                .map(|u| u.postos())
                .unwrap_or_default()
                .into_iter()
                .map(|p| format!("{}={}", p.id, p.nome))
                .collect();
            for (p, m) in self.sugestoes.iter().zip(self.escolhidos.iter_mut()) {
                let posto = nomes
                    .iter()
                    .find_map(|n| n.strip_prefix(&format!("{}=", p.posto)))
                    .unwrap_or("?");
                ui.checkbox(m, format!("[{posto}] {}: {}", p.descricao, p.circunstancia));
            }
            if ui
                .button(tx("exposicao-gravar_os_perigos_marcados"))
                .clicked()
            {
                let marcados: Vec<Perigo> = self
                    .sugestoes
                    .iter()
                    .zip(&self.escolhidos)
                    .filter(|(_, m)| **m)
                    .map(|(p, _)| p.clone())
                    .collect();
                let mut n = 0;
                if let Some(u) = s.unidade.as_mut() {
                    for p in &marcados {
                        if u.gravar(PERIGOS, "", p).is_ok() {
                            n += 1;
                        }
                    }
                }
                s.mudou();
                s.avisar(fx(
                    "exposicao-perigos_gravados_avalie_probabilidade_e",
                    &[("n", vx(&n))],
                ));
                self.sugestoes.clear();
                self.escolhidos.clear();
            }
        }
    }
}
