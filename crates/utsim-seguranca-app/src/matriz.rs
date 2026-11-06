//! Tela da matriz de risco: grade colorida, nomes e decisões dos níveis, importação e
//! exportação por CSV e gravação na unidade.

use eframe::egui::{self, RichText};
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::Matriz;
use utsim_seguranca_core::inventario::{Inventario, gravar_matriz};

#[derive(Default)]
pub struct TelaMatriz {
    /// Matriz em edição (carregada da unidade na primeira vez).
    pub rascunho: Option<Matriz>,
    pub arquivo_csv: String,
    /// Erros da última importação ou da validação ao gravar.
    pub erros: Vec<String>,
}

impl TelaMatriz {
    /// Importa o CSV de `arquivo_csv`; os erros de `de_csv` ficam em `erros`.
    pub fn importar(&mut self) {
        let atual = self.rascunho.clone().unwrap_or_default();
        self.erros.clear();
        match std::fs::read_to_string(self.arquivo_csv.trim()) {
            Err(e) => self.erros.push(format!("{}: {e}", self.arquivo_csv.trim())),
            Ok(t) => match Matriz::de_csv(&t, &atual) {
                Ok(m) => self.rascunho = Some(m),
                Err(e) => self.erros.push(e),
            },
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        let Some(u) = s.unidade.as_ref() else {
            ui.label("Abra uma unidade na tela inicial.");
            return;
        };
        if self.rascunho.is_none() {
            self.rascunho = Some(Inventario::ler(u).matriz);
        }
        ui.heading("Matriz de risco");
        ui.label(
            "Clique numa célula para trocar o nível dela. A matriz precisa ser monótona: mais \
             probabilidade ou mais severidade nunca baixa o nível.",
        );
        let m = self.rascunho.as_mut().expect("rascunho");
        ui.horizontal(|ui| {
            ui.label("Nome da matriz:");
            ui.text_edit_singleline(&mut m.nome);
        });
        // Grade: probabilidade maior em cima.
        let nn = m.niveis.len().max(1);
        egui::Grid::new("grade_matriz")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                ui.label("");
                for d in &m.severidade {
                    ui.strong(&d.nome);
                }
                ui.end_row();
                for p in (0..m.probabilidade.len()).rev() {
                    ui.strong(&m.probabilidade[p].nome);
                    for sv in 0..m.severidade.len() {
                        let atual = m
                            .celulas
                            .get(p)
                            .and_then(|l| l.get(sv))
                            .copied()
                            .unwrap_or(0);
                        let nivel = &m.niveis[atual.min(nn - 1)];
                        let cor = util::cor(&nivel.cor);
                        let b = egui::Button::new(
                            RichText::new(&nivel.nome).color(util::texto_sobre(cor)),
                        )
                        .fill(cor)
                        .min_size(egui::vec2(96.0, 28.0));
                        if ui.add(b).on_hover_text(&nivel.decisao).clicked()
                            && let Some(c) = m.celulas.get_mut(p).and_then(|l| l.get_mut(sv))
                        {
                            *c = (atual + 1) % nn;
                        }
                    }
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.collapsing("Níveis: nomes, cores e decisões", |ui| {
            egui::Grid::new("niveis").striped(true).show(ui, |ui| {
                ui.strong("Nível");
                ui.strong("Cor (#rrggbb)");
                ui.strong("Decisão da empresa");
                ui.end_row();
                for n in &mut m.niveis {
                    ui.text_edit_singleline(&mut n.nome);
                    ui.add(egui::TextEdit::singleline(&mut n.cor).desired_width(80.0));
                    ui.add(egui::TextEdit::singleline(&mut n.decisao).desired_width(420.0));
                    ui.end_row();
                }
            });
        });
        ui.collapsing("Degraus de probabilidade e severidade", |ui| {
            egui::Grid::new("degraus").striped(true).show(ui, |ui| {
                ui.strong("Degrau");
                ui.strong("Critério");
                ui.end_row();
                for (rot, lista) in [("P", &mut m.probabilidade), ("S", &mut m.severidade)] {
                    for (i, d) in lista.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{rot}{}", i + 1));
                            ui.add(egui::TextEdit::singleline(&mut d.nome).desired_width(110.0));
                        });
                        ui.add(egui::TextEdit::singleline(&mut d.criterio).desired_width(520.0));
                        ui.end_row();
                    }
                }
            });
        });
        let problemas = m.problemas();
        for p in &problemas {
            ui.colored_label(egui::Color32::from_rgb(230, 90, 80), p);
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Arquivo CSV:");
            ui.add(egui::TextEdit::singleline(&mut self.arquivo_csv).desired_width(420.0));
        });
        let tem_arquivo = !self.arquivo_csv.trim().is_empty();
        let mut importar = false;
        let mut exportar = false;
        let mut gravar = false;
        let mut recarregar = false;
        ui.horizontal(|ui| {
            importar = ui
                .add_enabled(tem_arquivo, egui::Button::new("Importar CSV"))
                .clicked();
            exportar = ui
                .add_enabled(tem_arquivo, egui::Button::new("Exportar CSV"))
                .clicked();
            gravar = ui
                .add_enabled(problemas.is_empty(), egui::Button::new("Gravar na unidade"))
                .clicked();
            recarregar = ui.button("Recarregar da unidade").clicked();
        });
        if importar {
            self.importar();
        }
        if exportar && let Some(m) = &self.rascunho {
            match std::fs::write(self.arquivo_csv.trim(), m.para_csv()) {
                Ok(()) => s.avisar("Matriz exportada."),
                Err(e) => s.falhar(e.to_string()),
            }
        }
        if gravar && let (Some(m), Some(u)) = (self.rascunho.clone(), s.unidade.as_mut()) {
            self.erros.clear();
            let r = gravar_matriz(u, &m);
            if let Err(e) = &r {
                self.erros.push(e.to_string());
            }
            s.resultado(r, "Matriz gravada na unidade.");
        }
        if recarregar && let Some(u) = s.unidade.as_ref() {
            self.rascunho = Some(Inventario::ler(u).matriz);
            self.erros.clear();
        }
        for e in &self.erros {
            ui.colored_label(egui::Color32::from_rgb(230, 90, 80), e);
        }
    }
}
