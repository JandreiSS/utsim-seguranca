//! Tela do checklist de proteção de máquina: inspeção item a item, modelos próprios ou
//! importados por CSV e as não conformidades levadas ao inventário de riscos.

use crate::texto::{fx, tx, vx};
use eframe::egui::{self, Color32};
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::checklist::{
    AVISO_NR12, INSPECOES, InspecaoMaquina, MODELOS, Modelo, Resposta, RespostaItem, gravar_perigos,
};

const VERMELHO: Color32 = Color32::from_rgb(230, 90, 80);

pub struct TelaChecklist {
    /// Id do posto escolhido.
    pub posto: String,
    pub maquina: String,
    /// O modelo em uso.
    pub modelo: Modelo,
    /// CSV de um checklist da empresa e o nome dele.
    pub arquivo_csv: String,
    pub nome_modelo: String,
    /// A inspeção em andamento (só vai para a unidade ao clicar em Gravar).
    pub inspecao: Option<InspecaoMaquina>,
}

impl Default for TelaChecklist {
    fn default() -> Self {
        TelaChecklist {
            posto: String::new(),
            maquina: String::new(),
            modelo: Modelo::proprio(),
            arquivo_csv: String::new(),
            nome_modelo: String::new(),
            inspecao: None,
        }
    }
}

fn nome_resposta(r: Resposta) -> &'static str {
    match r {
        Resposta::SemResposta => tx("checklist-sem_resposta"),
        Resposta::Conforme => tx("checklist-conforme"),
        Resposta::NaoConforme => tx("checklist-nao_conforme"),
        Resposta::NaoSeAplica => tx("checklist-nao_se_aplica"),
    }
}

impl TelaChecklist {
    /// Modelos importados e guardados na unidade.
    pub fn guardados(s: &Sessao) -> Vec<Modelo> {
        s.unidade
            .as_ref()
            .map(|u| u.estado().lista(MODELOS))
            .unwrap_or_default()
    }

    pub fn inspecoes(s: &Sessao) -> Vec<InspecaoMaquina> {
        let mut v: Vec<InspecaoMaquina> = s
            .unidade
            .as_ref()
            .map(|u| u.estado().lista(INSPECOES))
            .unwrap_or_default();
        v.sort_by(|a, b| b.data.cmp(&a.data).then(a.id.cmp(&b.id)));
        v
    }

    /// Remove uma inspeção gravada; se era a aberta, ela fecha.
    pub fn remover_inspecao(&mut self, s: &mut Sessao, id: &str) {
        if let Some(u) = s.unidade.as_mut() {
            let r = u.remover(INSPECOES, id);
            s.resultado(r, tx("checklist-inspecao_removida"));
        }
        if self.inspecao.as_ref().is_some_and(|i| i.id == id) {
            self.inspecao = None;
        }
    }

    pub fn usar_proprio(&mut self) {
        self.modelo = Modelo::proprio();
    }

    /// Lê o CSV do checklist da empresa e passa a usá-lo (ainda não guardado).
    pub fn importar_csv(&mut self, s: &mut Sessao) -> bool {
        let c = self.arquivo_csv.trim().to_string();
        let texto = match std::fs::read_to_string(&c) {
            Ok(t) => t,
            Err(e) => {
                s.falhar(format!("{c}: {e}"));
                return false;
            }
        };
        let nome = if self.nome_modelo.trim().is_empty() {
            tx("checklist-checklist_da_empresa").to_string()
        } else {
            self.nome_modelo.trim().to_string()
        };
        match Modelo::de_csv(&nome, &texto) {
            Ok(m) => {
                s.avisar(fx(
                    "checklist-item_ns_lido_s_guarde",
                    &[("itens", vx(&m.itens.len()))],
                ));
                self.modelo = m;
                true
            }
            Err(e) => {
                s.falhar(e);
                false
            }
        }
    }

    /// Guarda o modelo em uso na coleção de modelos da unidade.
    pub fn guardar_modelo(&mut self, s: &mut Sessao) -> bool {
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(MODELOS, &self.modelo.id, &self.modelo);
        match s.resultado(r, tx("checklist-modelo_de_checklist_guardado")) {
            Some(id) => {
                self.modelo.id = id;
                true
            }
            None => false,
        }
    }

    /// Começa uma inspeção do posto e da máquina escolhidos.
    pub fn nova(&mut self, s: &mut Sessao) -> bool {
        if self.posto.is_empty() || self.maquina.trim().is_empty() {
            s.falhar(tx("checklist-escolha_o_posto_e_informe"));
            return false;
        }
        let responsavel = s
            .unidade
            .as_ref()
            .map(|u| u.autor().to_string())
            .unwrap_or_default();
        self.inspecao = Some(InspecaoMaquina {
            posto: self.posto.clone(),
            maquina: self.maquina.trim().to_string(),
            modelo: if self.modelo.id.is_empty() {
                self.modelo.nome.clone()
            } else {
                self.modelo.id.clone()
            },
            data: util::hoje(),
            responsavel,
            ..InspecaoMaquina::default()
        });
        true
    }

    pub fn responder(&mut self, item: &str, r: Resposta) {
        if let Some(i) = self.inspecao.as_mut() {
            i.respostas.entry(item.to_string()).or_default().resposta = r;
        }
    }

    /// Abre uma inspeção gravada, com o modelo dela quando está na unidade.
    pub fn abrir(&mut self, s: &Sessao, insp: InspecaoMaquina) {
        let modelo = if insp.modelo == "proprio" {
            Some(Modelo::proprio())
        } else {
            Self::guardados(s)
                .into_iter()
                .find(|m| m.id == insp.modelo || m.nome == insp.modelo)
        };
        if let Some(m) = modelo {
            self.modelo = m;
        }
        self.posto = insp.posto.clone();
        self.maquina = insp.maquina.clone();
        self.inspecao = Some(insp);
    }

    pub fn gravar(&mut self, s: &mut Sessao) -> bool {
        let Some(i) = self.inspecao.as_mut() else {
            return false;
        };
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        match u.gravar(INSPECOES, &i.id.clone(), i) {
            Ok(id) => {
                i.id = id;
                s.mudou();
                s.avisar(tx("checklist-inspecao_gravada"));
                true
            }
            Err(e) => {
                s.falhar(e.to_string());
                false
            }
        }
    }

    /// Leva as não conformidades ao inventário de riscos do posto (a inspeção já gravada).
    /// Devolve quantos perigos foram criados.
    pub fn levar_ao_inventario(&mut self, s: &mut Sessao) -> Option<usize> {
        let i = self.inspecao.as_ref()?;
        if i.id.is_empty() {
            s.falhar(tx("checklist-grave_a_inspecao_antes_de"));
            return None;
        }
        let u = s.unidade.as_mut()?;
        let r = gravar_perigos(u, i, &self.modelo);
        match r {
            Ok(n) => {
                s.mudou();
                s.avisar(fx("checklist-perigo_s_novo_s_no", &[("n", vx(&n))]));
                Some(n)
            }
            Err(e) => {
                s.falhar(e.to_string());
                None
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        let Some(u) = s.unidade.as_ref() else {
            ui.label(tx("checklist-abra_uma_unidade_na_tela"));
            return;
        };
        let postos = u.postos();
        ui.heading(tx("checklist-checklist_de_protecao_de_maquina"));
        ui.colored_label(Color32::from_rgb(230, 160, 40), AVISO_NR12);
        if postos.is_empty() {
            ui.label(tx("checklist-crie_ou_importe_postos_na"));
            return;
        }
        if !postos.iter().any(|p| p.id == self.posto) {
            self.posto = postos[0].id.clone();
        }

        // Modelo do checklist.
        ui.separator();
        ui.strong(tx("checklist-modelo_do_checklist"));
        ui.horizontal(|ui| {
            ui.label(fx(
                "checklist-em_uso_itens",
                &[
                    ("nome", vx(&self.modelo.nome)),
                    ("itens", vx(&self.modelo.itens.len())),
                ],
            ));
            if ui
                .button(tx("checklist-usar_o_checklist_proprio"))
                .clicked()
            {
                self.usar_proprio();
            }
            for m in Self::guardados(s) {
                if ui
                    .button(fx("checklist-usar", &[("nome", vx(&m.nome))]))
                    .clicked()
                {
                    self.modelo = m;
                }
            }
        });
        // Importar outro modelo fica recolhido: quem começa usa o checklist próprio.
        egui::CollapsingHeader::new(tx("checklist-importar_outro_modelo"))
            .id_salt("importar_modelo_checklist")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tx("checklist-importar_csv_tema_item_severidade"));
                    utsim_ui_tema::arquivos::campo(
                        ui,
                        &mut self.arquivo_csv,
                        utsim_ui_tema::arquivos::Tipo::Abrir,
                        tx("checklist-escolha_o_arquivo_csv"),
                        &[utsim_ui_tema::arquivos::CSV],
                        300.0,
                    );
                    ui.label(tx("checklist-nome"));
                    ui.add(egui::TextEdit::singleline(&mut self.nome_modelo).desired_width(160.0));
                    if ui.button(tx("checklist-ler")).clicked() {
                        self.importar_csv(s);
                    }
                    if ui.button(tx("checklist-guardar_na_unidade")).clicked() {
                        self.guardar_modelo(s);
                    }
                })
            });

        // Inspeções gravadas.
        ui.separator();
        ui.strong(tx("checklist-inspecoes_gravadas"));
        let gravadas = Self::inspecoes(s);
        if gravadas.is_empty() {
            ui.weak(tx("checklist-nenhuma_inspecao_ainda"));
        }
        let (mut abrir, mut tirar) = (None, None);
        for g in &gravadas {
            ui.horizontal(|ui| {
                let nome = postos
                    .iter()
                    .find(|p| p.id == g.posto)
                    .map_or(g.posto.as_str(), |p| p.nome.as_str());
                ui.label(format!("{} · {} · {}", g.data, nome, g.maquina));
                if ui.small_button(tx("checklist-abrir")).clicked() {
                    abrir = Some(g.clone());
                }
                if utsim_ui_tema::campos::botao_confirmado(
                    ui,
                    tx("checklist-remover"),
                    ("inspecao", &g.id),
                ) {
                    tirar = Some(g.id.clone());
                }
            });
        }
        if let Some(g) = abrir {
            self.abrir(s, g);
        }
        if let Some(id) = tirar {
            self.remover_inspecao(s, &id);
        }

        // Nova inspeção.
        ui.separator();
        ui.strong(tx("checklist-nova_inspecao"));
        ui.horizontal(|ui| {
            ui.label(tx("checklist-posto"));
            let nome = postos
                .iter()
                .find(|p| p.id == self.posto)
                .map_or("?", |p| p.nome.as_str());
            egui::ComboBox::from_id_salt("posto_checklist")
                .selected_text(nome)
                .show_ui(ui, |ui| {
                    for p in &postos {
                        ui.selectable_value(&mut self.posto, p.id.clone(), &p.nome);
                    }
                });
            ui.label(tx("checklist-maquina"));
            ui.add(egui::TextEdit::singleline(&mut self.maquina).desired_width(180.0));
            if ui.button(tx("checklist-nova_inspecao")).clicked() {
                self.nova(s);
            }
        });

        let Some(insp) = self.inspecao.as_mut() else {
            return;
        };
        ui.label(fx(
            "checklist-inspecao_de_em_por",
            &[
                ("maquina", vx(&insp.maquina)),
                ("data", vx(&insp.data)),
                ("responsavel", vx(&insp.responsavel)),
            ],
        ));
        let mut tema_atual = String::new();
        for item in &self.modelo.itens {
            if item.tema != tema_atual {
                ui.add_space(4.0);
                ui.strong(&item.tema);
                tema_atual = item.tema.clone();
            }
            let r = insp
                .respostas
                .entry(item.id.clone())
                .or_insert_with(RespostaItem::default);
            ui.group(|ui| {
                ui.label(&item.texto);
                ui.horizontal(|ui| {
                    for x in [
                        Resposta::Conforme,
                        Resposta::NaoConforme,
                        Resposta::NaoSeAplica,
                    ] {
                        ui.selectable_value(&mut r.resposta, x, nome_resposta(x));
                    }
                    if r.resposta == Resposta::NaoConforme {
                        ui.colored_label(VERMELHO, tx("checklist-vira_perigo_do_posto"));
                    }
                });
                if r.resposta != Resposta::SemResposta {
                    ui.horizontal(|ui| {
                        ui.label(tx("checklist-observacao"));
                        ui.add(egui::TextEdit::singleline(&mut r.observacao).desired_width(260.0));
                        ui.label(tx("checklist-acao"));
                        ui.add(egui::TextEdit::singleline(&mut r.acao).desired_width(260.0));
                    });
                }
            });
        }
        let pendentes = insp.pendentes(&self.modelo).len();
        let nao_conformes = insp.nao_conformes(&self.modelo).len();
        ui.label(fx(
            "checklist-item_ns_sem_resposta_nao",
            &[
                ("pendentes", vx(&pendentes)),
                ("nao_conformes", vx(&nao_conformes)),
            ],
        ));
        let (mut gravar, mut levar) = (false, false);
        ui.horizontal(|ui| {
            gravar = ui.button(tx("checklist-gravar_inspecao")).clicked();
            levar = ui
                .add_enabled(
                    nao_conformes > 0,
                    egui::Button::new(tx("checklist-levar_nao_conformidades_ao_inventario")),
                )
                .clicked();
        });
        if gravar {
            self.gravar(s);
        }
        if levar {
            self.levar_ao_inventario(s);
        }
    }
}
