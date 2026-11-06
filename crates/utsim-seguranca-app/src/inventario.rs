//! Tela do inventário de riscos por posto, nas cinco etapas de `ETAPAS`.

use std::collections::BTreeMap;

use eframe::egui::{self, Color32, RichText};
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::inventario::{
    ACOES, Acao, ETAPAS, Grupo, Hierarquia, Inventario, PERIGOS, Perigo, REAVALIACOES, Reavaliacao,
    Situacao,
};
use utsim_seguranca_core::matriz::{Degrau, Matriz};

const VERMELHO: Color32 = Color32::from_rgb(230, 90, 80);

/// Operação pedida pela tela, aplicada depois de desenhar.
enum Cmd {
    GravarPerigo(Perigo),
    RemoverPerigo(String),
    GravarAcao(Acao),
    RemoverAcao(String),
    GravarReav(Reavaliacao),
}

#[derive(Default)]
pub struct TelaInventario {
    /// Id do posto escolhido.
    pub posto: String,
    /// Etapa de 0 a 4 (`ETAPAS`).
    pub etapa: usize,
    perigos: BTreeMap<String, (Perigo, Perigo)>,
    acoes: BTreeMap<String, (Acao, Acao)>,
    pub novo_perigo: Option<Perigo>,
    /// Ação nova por id de perigo.
    novas_acoes: BTreeMap<String, Acao>,
    /// Reavaliação nova por id de perigo.
    novas_reav: BTreeMap<String, Reavaliacao>,
}

fn combo_degrau(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    valor: &mut usize,
    d: &[Degrau],
) {
    let texto = d.get(*valor).map_or_else(
        || "?".to_string(),
        |x| format!("{} ({})", x.nome, *valor + 1),
    );
    egui::ComboBox::from_id_salt(id)
        .selected_text(texto)
        .show_ui(ui, |ui| {
            for (i, x) in d.iter().enumerate() {
                ui.selectable_value(valor, i, format!("{} ({})", x.nome, i + 1))
                    .on_hover_text(&x.criterio);
            }
        });
}

/// Nome do nível com a cor dele.
fn selo(ui: &mut egui::Ui, m: &Matriz, p: usize, s: usize) {
    let n = m.nivel_de(p, s);
    let cor = util::cor(&n.cor);
    ui.label(
        RichText::new(format!(" {} ", n.nome))
            .background_color(cor)
            .color(util::texto_sobre(cor)),
    )
    .on_hover_text(&n.decisao);
}

fn campo(ui: &mut egui::Ui, rotulo: &str, texto: &mut String) {
    ui.horizontal(|ui| {
        ui.label(rotulo);
        ui.add(egui::TextEdit::singleline(texto).desired_width(f32::INFINITY));
    });
}

fn combo_grupo(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, g: &mut Grupo) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(g.nome())
        .show_ui(ui, |ui| {
            for x in Grupo::TODOS {
                ui.selectable_value(g, x, x.nome());
            }
        });
}

fn editor_acao(ui: &mut egui::Ui, a: &mut Acao, atrasada: bool) {
    ui.horizontal(|ui| {
        ui.label("Hierarquia");
        egui::ComboBox::from_id_salt(("hier", &a.id, &a.perigo))
            .selected_text(a.hierarquia.nome())
            .show_ui(ui, |ui| {
                for h in Hierarquia::TODAS {
                    ui.selectable_value(&mut a.hierarquia, h, h.nome());
                }
            });
        ui.label("Situação");
        egui::ComboBox::from_id_salt(("sit", &a.id, &a.perigo))
            .selected_text(a.situacao.nome())
            .show_ui(ui, |ui| {
                for x in Situacao::TODAS {
                    ui.selectable_value(&mut a.situacao, x, x.nome());
                }
            });
        ui.label("Eficaz");
        let t = match a.eficaz {
            None => "não verificada",
            Some(true) => "sim",
            Some(false) => "não",
        };
        egui::ComboBox::from_id_salt(("efi", &a.id, &a.perigo))
            .selected_text(t)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut a.eficaz, None, "não verificada");
                ui.selectable_value(&mut a.eficaz, Some(true), "sim");
                ui.selectable_value(&mut a.eficaz, Some(false), "não");
            });
    });
    campo(ui, "Descrição", &mut a.descricao);
    ui.horizontal(|ui| {
        ui.label("Responsável");
        ui.add(egui::TextEdit::singleline(&mut a.responsavel).desired_width(200.0));
        ui.label("Prazo (AAAA-MM-DD)");
        ui.add(egui::TextEdit::singleline(&mut a.prazo).desired_width(100.0));
        if atrasada {
            ui.colored_label(VERMELHO, "ATRASADA");
        }
    });
}

impl TelaInventario {
    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        let Some(u) = s.unidade.as_ref() else {
            ui.label("Abra uma unidade na tela inicial.");
            return;
        };
        let inv = Inventario::ler(u);
        ui.heading("Inventário de riscos por posto");
        if inv.postos.is_empty() {
            ui.label("A unidade ainda não tem postos. Crie-os na tela Postos.");
            return;
        }
        if !inv.postos.iter().any(|p| p.id == self.posto) {
            self.posto = inv.postos[0].id.clone();
        }
        self.etapa = self.etapa.min(ETAPAS.len() - 1);
        let hoje = util::hoje();

        ui.horizontal_wrapped(|ui| {
            for (i, e) in ETAPAS.iter().enumerate() {
                ui.selectable_value(&mut self.etapa, i, *e);
            }
        });
        let nome_posto = inv
            .postos
            .iter()
            .find(|p| p.id == self.posto)
            .map(|p| p.nome.clone())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("Posto:");
            egui::ComboBox::from_id_salt("posto_inventario")
                .selected_text(&nome_posto)
                .show_ui(ui, |ui| {
                    for p in &inv.postos {
                        ui.selectable_value(&mut self.posto, p.id.clone(), &p.nome);
                    }
                });
        });

        let meus: Vec<Perigo> = inv
            .perigos_do_posto(&self.posto)
            .into_iter()
            .cloned()
            .collect();
        let ids: Vec<&str> = meus.iter().map(|p| p.id.as_str()).collect();
        let sem_acao: Vec<&Perigo> = inv
            .sem_acao()
            .into_iter()
            .filter(|p| ids.contains(&p.id.as_str()))
            .collect();
        let atrasadas: Vec<&Acao> = inv
            .atrasadas(&hoje)
            .into_iter()
            .filter(|a| ids.contains(&a.perigo.as_str()))
            .collect();
        if !sem_acao.is_empty() {
            ui.colored_label(
                VERMELHO,
                format!(
                    "{} perigos acima do nível mais baixo sem ação: {}",
                    sem_acao.len(),
                    sem_acao
                        .iter()
                        .map(|p| p.descricao.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            );
        }
        if !atrasadas.is_empty() {
            ui.colored_label(
                VERMELHO,
                format!("{} ações atrasadas em {hoje}.", atrasadas.len()),
            );
        }
        ui.separator();

        let mut cmds: Vec<Cmd> = Vec::new();
        match self.etapa {
            0 => self.etapa_identificar(ui, &inv),
            1 => self.etapa_perigos(ui, &meus, &mut cmds),
            2 => self.etapa_avaliar(ui, &inv, &meus, &mut cmds),
            3 => self.etapa_acoes(ui, &inv, &meus, &hoje, &mut cmds),
            _ => self.etapa_acompanhar(ui, &inv, &meus, &hoje, &mut cmds),
        }
        for c in cmds {
            self.aplicar(s, c);
        }
    }

    fn etapa_identificar(&self, ui: &mut egui::Ui, inv: &Inventario) {
        let Some(p) = inv.postos.iter().find(|p| p.id == self.posto) else {
            return;
        };
        ui.strong(&p.nome);
        egui::Grid::new("identificar")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Área");
                ui.label(&p.area);
                ui.end_row();
                ui.label("Descrição");
                ui.label(&p.descricao);
                ui.end_row();
                ui.label("Pessoas expostas por turno");
                ui.label(p.pessoas_expostas.to_string());
                ui.end_row();
                ui.label("EPI exigidos");
                ui.label(p.epi.join(", "));
                ui.end_row();
                ui.label("Atividades");
                ui.label(p.atividades.join("; "));
                ui.end_row();
            });
        ui.weak("Para mudar estes dados, use a tela Postos.");
    }

    fn etapa_perigos(&mut self, ui: &mut egui::Ui, meus: &[Perigo], cmds: &mut Vec<Cmd>) {
        for p in meus {
            let d = rascunho(&mut self.perigos, &p.id, p);
            let sujo = d != p;
            let titulo = format!(
                "[{}] {}{}",
                d.grupo.nome(),
                d.descricao,
                if sujo { " *" } else { "" }
            );
            egui::CollapsingHeader::new(titulo)
                .id_salt(("perigo1", &p.id))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Grupo");
                        combo_grupo(ui, ("grupo", &p.id), &mut d.grupo);
                        if !d.origem.is_empty() {
                            ui.weak(format!("origem: {}", d.origem));
                        }
                    });
                    campo(ui, "Descrição", &mut d.descricao);
                    campo(ui, "Fonte", &mut d.fonte);
                    campo(ui, "Circunstância", &mut d.circunstancia);
                    campo(ui, "Possíveis lesões", &mut d.lesoes);
                    botoes_perigo(ui, d, p, cmds);
                });
        }
        ui.separator();
        match &mut self.novo_perigo {
            None => {
                if ui.button("Novo perigo").clicked() {
                    self.novo_perigo = Some(Perigo {
                        posto: self.posto.clone(),
                        origem: "manual".into(),
                        ..Perigo::default()
                    });
                }
            }
            Some(n) => {
                ui.strong("Novo perigo");
                ui.horizontal(|ui| {
                    ui.label("Grupo");
                    combo_grupo(ui, "grupo_novo", &mut n.grupo);
                });
                campo(ui, "Descrição", &mut n.descricao);
                campo(ui, "Fonte", &mut n.fonte);
                campo(ui, "Circunstância", &mut n.circunstancia);
                campo(ui, "Possíveis lesões", &mut n.lesoes);
                let mut fechar = false;
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !n.descricao.trim().is_empty(),
                            egui::Button::new("Gravar perigo"),
                        )
                        .clicked()
                    {
                        cmds.push(Cmd::GravarPerigo(n.clone()));
                        fechar = true;
                    }
                    if ui.button("Cancelar").clicked() {
                        fechar = true;
                    }
                });
                if fechar {
                    self.novo_perigo = None;
                }
            }
        }
    }

    fn etapa_avaliar(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        if meus.is_empty() {
            ui.label("Sem perigos neste posto. Levante-os na etapa 2.");
        }
        for p in meus {
            let d = rascunho(&mut self.perigos, &p.id, p);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&d.descricao);
                    selo(ui, m, d.probabilidade, d.severidade);
                });
                campo(ui, "Controles existentes", &mut d.controles);
                ui.horizontal(|ui| {
                    ui.label("Probabilidade");
                    combo_degrau(ui, ("prob", &p.id), &mut d.probabilidade, &m.probabilidade);
                    ui.label("Severidade");
                    combo_degrau(ui, ("sev", &p.id), &mut d.severidade, &m.severidade);
                });
                botoes_perigo(ui, d, p, cmds);
            });
        }
    }

    fn etapa_acoes(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        hoje: &str,
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        let sem: Vec<&str> = inv.sem_acao().iter().map(|p| p.id.as_str()).collect();
        for p in meus {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&p.descricao);
                    selo(ui, m, p.probabilidade, p.severidade);
                    if sem.contains(&p.id.as_str()) {
                        ui.colored_label(VERMELHO, "SEM AÇÃO");
                    }
                });
                for a in inv.acoes_do_perigo(&p.id) {
                    let atrasada = inv.atrasadas(hoje).iter().any(|x| x.id == a.id);
                    let d = rascunho(&mut self.acoes, &a.id, a);
                    let sujo = d != a;
                    ui.separator();
                    editor_acao(ui, d, atrasada);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(sujo, egui::Button::new("Gravar ação"))
                            .clicked()
                        {
                            cmds.push(Cmd::GravarAcao(d.clone()));
                        }
                        if sujo && ui.button("Descartar mudanças").clicked() {
                            *d = a.clone();
                        }
                        if ui.button("Remover ação").clicked() {
                            cmds.push(Cmd::RemoverAcao(a.id.clone()));
                        }
                    });
                }
                match self.novas_acoes.get_mut(&p.id) {
                    None => {
                        if ui.button("Nova ação").clicked() {
                            self.novas_acoes.insert(
                                p.id.clone(),
                                Acao {
                                    perigo: p.id.clone(),
                                    ..Acao::default()
                                },
                            );
                        }
                    }
                    Some(n) => {
                        ui.separator();
                        ui.strong("Nova ação");
                        editor_acao(ui, n, false);
                        let mut fechar = false;
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    !n.descricao.trim().is_empty(),
                                    egui::Button::new("Gravar nova ação"),
                                )
                                .clicked()
                            {
                                cmds.push(Cmd::GravarAcao(n.clone()));
                                fechar = true;
                            }
                            if ui.button("Cancelar").clicked() {
                                fechar = true;
                            }
                        });
                        if fechar {
                            self.novas_acoes.remove(&p.id);
                        }
                    }
                }
            });
        }
    }

    fn etapa_acompanhar(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        hoje: &str,
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        for p in meus {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&p.descricao);
                    ui.label("avaliação inicial:");
                    selo(ui, m, p.probabilidade, p.severidade);
                    ui.label("nível atual:");
                    let atual = inv.nivel_atual(p);
                    let n = &m.niveis[atual.min(m.niveis.len() - 1)];
                    let cor = util::cor(&n.cor);
                    ui.label(
                        RichText::new(format!(" {} ", n.nome))
                            .background_color(cor)
                            .color(util::texto_sobre(cor)),
                    );
                });
                let mut hist: Vec<&Reavaliacao> = inv
                    .reavaliacoes
                    .iter()
                    .filter(|r| r.perigo == p.id)
                    .collect();
                hist.sort_by(|a, b| a.data.cmp(&b.data));
                for r in hist {
                    ui.horizontal(|ui| {
                        ui.label(&r.data);
                        selo(ui, m, r.probabilidade, r.severidade);
                        ui.label(&r.nota);
                    });
                }
                let n = self
                    .novas_reav
                    .entry(p.id.clone())
                    .or_insert_with(|| Reavaliacao {
                        perigo: p.id.clone(),
                        id: String::new(),
                        data: hoje.to_string(),
                        probabilidade: p.probabilidade,
                        severidade: p.severidade,
                        nota: String::new(),
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Data");
                    ui.add(egui::TextEdit::singleline(&mut n.data).desired_width(90.0));
                    ui.label("P");
                    combo_degrau(ui, ("rp", &p.id), &mut n.probabilidade, &m.probabilidade);
                    ui.label("S");
                    combo_degrau(ui, ("rs", &p.id), &mut n.severidade, &m.severidade);
                });
                campo(ui, "Nota", &mut n.nota);
                if ui.button("Registrar reavaliação").clicked() {
                    cmds.push(Cmd::GravarReav(n.clone()));
                }
            });
        }
    }

    fn aplicar(&mut self, s: &mut Sessao, c: Cmd) {
        let Some(u) = s.unidade.as_mut() else {
            return;
        };
        match c {
            Cmd::GravarPerigo(p) => {
                let r = u.gravar(PERIGOS, &p.id, &p);
                if let Some(id) = s.resultado(r, "Perigo gravado.") {
                    self.perigos.remove(&id);
                }
            }
            Cmd::RemoverPerigo(id) => {
                let r = u.remover(PERIGOS, &id);
                if s.resultado(r, "Perigo removido.").is_some() {
                    self.perigos.remove(&id);
                }
            }
            Cmd::GravarAcao(a) => {
                let r = u.gravar(ACOES, &a.id, &a);
                if let Some(id) = s.resultado(r, "Ação gravada.") {
                    self.acoes.remove(&id);
                }
            }
            Cmd::RemoverAcao(id) => {
                let r = u.remover(ACOES, &id);
                if s.resultado(r, "Ação removida.").is_some() {
                    self.acoes.remove(&id);
                }
            }
            Cmd::GravarReav(r0) => {
                let r = u.gravar(REAVALIACOES, "", &r0);
                if s.resultado(r, "Reavaliação registrada.").is_some() {
                    self.novas_reav.remove(&r0.perigo);
                }
            }
        }
    }
}

/// Rascunho de um registro: guarda a base lida e a cópia editada. Se a base mudou na pasta
/// e a pessoa não tinha editado nada, a cópia acompanha; se tinha editado, as edições ficam.
fn rascunho<'a, T: Clone + PartialEq>(
    m: &'a mut BTreeMap<String, (T, T)>,
    id: &str,
    atual: &T,
) -> &'a mut T {
    let e = m
        .entry(id.to_string())
        .or_insert_with(|| (atual.clone(), atual.clone()));
    if e.0 != *atual {
        if e.0 == e.1 {
            e.1 = atual.clone();
        }
        e.0 = atual.clone();
    }
    &mut e.1
}

fn botoes_perigo(ui: &mut egui::Ui, d: &mut Perigo, original: &Perigo, cmds: &mut Vec<Cmd>) {
    let sujo = d != original;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(sujo, egui::Button::new("Gravar perigo"))
            .clicked()
        {
            cmds.push(Cmd::GravarPerigo(d.clone()));
        }
        if sujo && ui.button("Descartar mudanças").clicked() {
            *d = original.clone();
        }
        if ui.button("Remover perigo").clicked() {
            cmds.push(Cmd::RemoverPerigo(original.id.clone()));
        }
    });
}
