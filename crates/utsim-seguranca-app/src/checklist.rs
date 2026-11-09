//! Tela do checklist de proteção de máquina: inspeção item a item, modelos próprios ou
//! importados por CSV e as não conformidades levadas ao inventário de riscos.

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
        Resposta::SemResposta => "sem resposta",
        Resposta::Conforme => "conforme",
        Resposta::NaoConforme => "não conforme",
        Resposta::NaoSeAplica => "não se aplica",
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
            "Checklist da empresa".to_string()
        } else {
            self.nome_modelo.trim().to_string()
        };
        match Modelo::de_csv(&nome, &texto) {
            Ok(m) => {
                s.avisar(format!(
                    "{} item(ns) lido(s). Guarde o modelo para usá-lo de novo.",
                    m.itens.len()
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
        match s.resultado(r, "Modelo de checklist guardado.") {
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
            s.falhar("Escolha o posto e informe a máquina.");
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
                s.avisar("Inspeção gravada.");
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
            s.falhar("Grave a inspeção antes de levar ao inventário.");
            return None;
        }
        let u = s.unidade.as_mut()?;
        let r = gravar_perigos(u, i, &self.modelo);
        match r {
            Ok(n) => {
                s.mudou();
                s.avisar(format!("{n} perigo(s) novo(s) no inventário."));
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
            ui.label("Abra uma unidade na tela inicial.");
            return;
        };
        let postos = u.postos();
        ui.heading("Checklist de proteção de máquina");
        ui.colored_label(Color32::from_rgb(230, 160, 40), AVISO_NR12);
        if postos.is_empty() {
            ui.label("Crie ou importe postos na tela Postos.");
            return;
        }
        if !postos.iter().any(|p| p.id == self.posto) {
            self.posto = postos[0].id.clone();
        }

        // Modelo do checklist.
        ui.separator();
        ui.strong("Modelo do checklist");
        ui.horizontal(|ui| {
            ui.label(format!(
                "Em uso: {} ({} itens)",
                self.modelo.nome,
                self.modelo.itens.len()
            ));
            if ui.button("Usar o checklist próprio").clicked() {
                self.usar_proprio();
            }
            for m in Self::guardados(s) {
                if ui.button(format!("Usar '{}'", m.nome)).clicked() {
                    self.modelo = m;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Importar CSV (tema;item;severidade):");
            utsim_ui_tema::arquivos::campo(
                ui,
                &mut self.arquivo_csv,
                utsim_ui_tema::arquivos::Tipo::Abrir,
                "Escolha o arquivo CSV",
                &[utsim_ui_tema::arquivos::CSV],
                300.0,
            );
            ui.label("Nome:");
            ui.add(egui::TextEdit::singleline(&mut self.nome_modelo).desired_width(160.0));
            if ui.button("Ler").clicked() {
                self.importar_csv(s);
            }
            if ui.button("Guardar na unidade").clicked() {
                self.guardar_modelo(s);
            }
        });

        // Inspeções gravadas.
        ui.separator();
        ui.strong("Inspeções gravadas");
        let gravadas = Self::inspecoes(s);
        if gravadas.is_empty() {
            ui.weak("Nenhuma inspeção ainda.");
        }
        let mut abrir = None;
        for g in &gravadas {
            ui.horizontal(|ui| {
                let nome = postos
                    .iter()
                    .find(|p| p.id == g.posto)
                    .map_or(g.posto.as_str(), |p| p.nome.as_str());
                ui.label(format!("{} · {} · {}", g.data, nome, g.maquina));
                if ui.small_button("Abrir").clicked() {
                    abrir = Some(g.clone());
                }
            });
        }
        if let Some(g) = abrir {
            self.abrir(s, g);
        }

        // Nova inspeção.
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Posto:");
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
            ui.label("Máquina:");
            ui.add(egui::TextEdit::singleline(&mut self.maquina).desired_width(180.0));
            if ui.button("Nova inspeção").clicked() {
                self.nova(s);
            }
        });

        let Some(insp) = self.inspecao.as_mut() else {
            return;
        };
        ui.label(format!(
            "Inspeção de {} em {} por {}",
            insp.maquina, insp.data, insp.responsavel
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
                        ui.colored_label(VERMELHO, "vira perigo do posto");
                    }
                });
                if r.resposta != Resposta::SemResposta {
                    ui.horizontal(|ui| {
                        ui.label("Observação");
                        ui.add(egui::TextEdit::singleline(&mut r.observacao).desired_width(260.0));
                        ui.label("Ação");
                        ui.add(egui::TextEdit::singleline(&mut r.acao).desired_width(260.0));
                    });
                }
            });
        }
        let pendentes = insp.pendentes(&self.modelo).len();
        let nao_conformes = insp.nao_conformes(&self.modelo).len();
        ui.label(format!(
            "{pendentes} item(ns) sem resposta · {nao_conformes} não conforme(s)"
        ));
        let (mut gravar, mut levar) = (false, false);
        ui.horizontal(|ui| {
            gravar = ui.button("Gravar inspeção").clicked();
            levar = ui
                .add_enabled(
                    nao_conformes > 0,
                    egui::Button::new("Levar não conformidades ao inventário"),
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
