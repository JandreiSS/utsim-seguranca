//! Tela do cadastro de equipamentos de proteção individual e das pendências.

use eframe::egui::{self, Color32};
use utsim_model::quality::PPE_NAMES;
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::checklist::{EPIS, Epi, pendencias_epi};

const VERMELHO: Color32 = Color32::from_rgb(230, 90, 80);

#[derive(Default)]
pub struct TelaEpi {
    pub edicao: Option<Epi>,
}

impl TelaEpi {
    pub fn lista(s: &Sessao) -> Vec<Epi> {
        let mut v: Vec<Epi> = s
            .unidade
            .as_ref()
            .map(|u| u.estado().lista(EPIS))
            .unwrap_or_default();
        v.sort_by(|a, b| a.tipo.cmp(&b.tipo).then(a.id.cmp(&b.id)));
        v
    }

    /// Pendências de hoje: EPI exigido nos postos e fora do cadastro, CA vencido.
    pub fn pendencias(s: &Sessao) -> Vec<String> {
        let Some(u) = s.unidade.as_ref() else {
            return Vec::new();
        };
        pendencias_epi(&u.postos(), &Self::lista(s), &util::hoje())
    }

    pub fn gravar(&mut self, s: &mut Sessao) -> bool {
        let Some(e) = self.edicao.take() else {
            return false;
        };
        if e.tipo.trim().is_empty() {
            s.falhar("Informe o tipo do equipamento.");
            self.edicao = Some(e);
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(EPIS, &e.id, &e);
        s.resultado(r, "EPI gravado.").is_some()
    }

    pub fn remover(&mut self, s: &mut Sessao, id: &str) {
        if let Some(u) = s.unidade.as_mut() {
            let r = u.remover(EPIS, id);
            s.resultado(r, "EPI removido.");
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        if s.unidade.is_none() {
            ui.label("Abra uma unidade na tela inicial.");
            return;
        }
        ui.heading("Equipamentos de proteção individual");
        let hoje = util::hoje();
        let lista = Self::lista(s);
        if lista.is_empty() {
            ui.weak("Nenhum EPI cadastrado.");
        } else {
            let mut editar = None;
            let mut tirar = None;
            egui::Grid::new("grade_epi").striped(true).show(ui, |ui| {
                for c in ["Tipo", "Uso", "CA", "Validade do CA", "Troca (dias)", ""] {
                    ui.strong(c);
                }
                ui.end_row();
                for e in &lista {
                    ui.label(&e.tipo);
                    ui.label(&e.uso);
                    ui.label(&e.ca);
                    if e.vencido(&hoje) {
                        ui.colored_label(VERMELHO, format!("{} (vencido)", e.validade_ca));
                    } else {
                        ui.label(&e.validade_ca);
                    }
                    ui.label(e.troca_dias.map_or("–".into(), |d| d.to_string()));
                    ui.horizontal(|ui| {
                        if ui.small_button("Editar").clicked() {
                            editar = Some(e.clone());
                        }
                        if ui.small_button("Remover").clicked() {
                            tirar = Some(e.id.clone());
                        }
                    });
                    ui.end_row();
                }
            });
            if let Some(e) = editar {
                self.edicao = Some(e);
            }
            if let Some(id) = tirar {
                self.remover(s, &id);
            }
        }
        if ui.button("Novo EPI").clicked() {
            self.edicao = Some(Epi::default());
        }
        let (mut gravar, mut cancelar) = (false, false);
        if let Some(e) = self.edicao.as_mut() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Tipo");
                    egui::ComboBox::from_id_salt("tipo_epi")
                        .selected_text(if e.tipo.is_empty() {
                            "escolha"
                        } else {
                            &e.tipo
                        })
                        .show_ui(ui, |ui| {
                            for n in PPE_NAMES {
                                ui.selectable_value(&mut e.tipo, n.to_string(), *n);
                            }
                        });
                    ui.label("ou outro");
                    ui.add(egui::TextEdit::singleline(&mut e.tipo).desired_width(140.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Descrição");
                    ui.add(egui::TextEdit::singleline(&mut e.descricao).desired_width(300.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Uso (contra o quê e quando)");
                    ui.add(egui::TextEdit::singleline(&mut e.uso).desired_width(300.0));
                });
                ui.horizontal(|ui| {
                    ui.label("CA (número digitado, o programa não confere)");
                    ui.add(egui::TextEdit::singleline(&mut e.ca).desired_width(100.0));
                    ui.label("Validade do CA (AAAA-MM-DD)");
                    ui.add(egui::TextEdit::singleline(&mut e.validade_ca).desired_width(100.0));
                });
                ui.horizontal(|ui| {
                    let mut tem = e.troca_dias.is_some();
                    ui.checkbox(&mut tem, "intervalo de troca (dias)");
                    match (tem, e.troca_dias.is_some()) {
                        (true, false) => e.troca_dias = Some(180),
                        (false, true) => e.troca_dias = None,
                        _ => {}
                    }
                    if let Some(d) = e.troca_dias.as_mut() {
                        ui.add(egui::DragValue::new(d).range(1..=3650));
                    }
                });
                ui.horizontal(|ui| {
                    gravar = ui.button("Gravar EPI").clicked();
                    cancelar = ui.button("Cancelar").clicked();
                });
            });
        }
        if gravar {
            self.gravar(s);
        }
        if cancelar {
            self.edicao = None;
        }

        ui.separator();
        ui.strong("Pendências");
        let p = Self::pendencias(s);
        if p.is_empty() {
            ui.label("Nenhuma pendência.");
        }
        for t in &p {
            ui.colored_label(VERMELHO, t);
        }
    }
}
