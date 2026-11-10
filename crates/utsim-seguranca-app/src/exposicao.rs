//! Tela de exposição simulada: tempo em zona, entradas sem EPI e quase-conflitos.

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
    ui.label(format!("{:.2} ± {:.2}", i.mean, i.half_width));
    ui.end_row();
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
        s.avisar("Calculando a exposição...");
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
            ui.label("Abra uma unidade na tela inicial.");
            return;
        }
        if self.trabalho.sondar(ui.ctx()) {
            match self.trabalho.erro() {
                Some(e) => s.falhar(e.to_string()),
                None => s.avisar("Exposição calculada."),
            }
        }
        ui.heading("Exposição simulada");
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
            ui.label("Modelo (.ron):");
            utsim_posto_ui::util::campo_modelo(ui, &mut self.modelo, s.unidade.as_ref(), 420.0);
            if !modelos.is_empty() {
                egui::ComboBox::from_id_salt("modelos_exposicao")
                    .selected_text("modelos dos postos")
                    .show_ui(ui, |ui| {
                        for m in &modelos {
                            ui.selectable_value(&mut self.modelo, m.clone(), m);
                        }
                    });
            }
            let livre = !self.trabalho.ocupado() && !self.modelo.trim().is_empty();
            if ui.add_enabled(livre, egui::Button::new("Rodar")).clicked() {
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
        ui.strong(format!(
            "{} replicações · Origem: Simulado",
            r.replicacoes.len()
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
        egui::Grid::new("exp_resumo").striped(true).show(ui, |ui| {
            ui.strong("Indicador");
            ui.strong("Média ± IC 95 %");
            ui.end_row();
            media(ui, "Tempo das pessoas em zona de risco (s)", &tempo);
            media(ui, "Entradas em zona sem o EPI pedido", &sem_epi);
            media(ui, "Quase-conflitos simulados", &quase);
        });
        if let Some(e0) = r.replicacoes.first() {
            ui.add_space(4.0);
            ui.strong("Por zona e pessoa (primeira replicação)");
            egui::Grid::new("exp_zonas").striped(true).show(ui, |ui| {
                for c in [
                    "Zona",
                    "Pessoa",
                    "Tempo (s)",
                    "Entradas",
                    "Sem EPI",
                    "EPI faltante",
                ] {
                    ui.strong(c);
                }
                ui.end_row();
                for z in &e0.zones {
                    ui.label(&z.zone);
                    ui.label(&z.person);
                    ui.label(format!("{:.1}", z.time));
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
            sugerir = ui.button("Sugerir perigos").clicked();
            gravar_html = ui.button("Gravar HTML de exposição").clicked();
        });
        if gravar_html {
            let html = rel::html(&r.modelo, &r.replicacoes);
            if let Some(u) = s.unidade.as_ref() {
                match u.gravar_relatorio("exposicao.html", html.as_bytes()) {
                    Ok(p) => s.avisar(format!("Relatório gravado em {}", p.display())),
                    Err(e) => s.falhar(e.to_string()),
                }
            }
        }
        if sugerir {
            self.sugerir(s);
            let n = self.sugestoes.len();
            s.avisar(format!("{n} perigos sugeridos (nenhum gravado ainda)."));
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
            if ui.button("Gravar os perigos marcados").clicked() {
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
                s.avisar(format!(
                    "{n} perigos gravados. Avalie probabilidade e severidade no inventário."
                ));
                self.sugestoes.clear();
                self.escolhidos.clear();
            }
        }
    }
}
