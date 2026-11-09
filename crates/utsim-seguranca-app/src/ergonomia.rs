//! Tela de ergonomia: estudo de tempos de um modelo, marcações de postura, carga, pega e
//! vibração por elemento, nota ao vivo com o método da unidade e comparação de dois modelos.

use eframe::egui;
use utsim_diagram::rastro::capturar;
use utsim_diagram::tempos::{Estudo, Opcoes, estudar};
use utsim_posto::Unidade;
use utsim_posto_ui::{Sessao, Trabalho, util};
use utsim_seguranca_core::ergonomia::{
    AVISO_NR17, Marcacao, Metodo, NotaPosto, PEGA, POSTURA, VIBRACAO, avaliar, comparar, html,
};

/// Coleção das marcações, id = ator, elemento e modelo.
pub const MARCACOES: &str = "marcacoes_ergonomia";
/// Coleção da configuração de segurança.
pub const CONFIG: &str = "config_seguranca";
/// Id do método de ergonomia da empresa na coleção de configuração.
pub const ID_METODO: &str = "metodo_ergonomia";

const UNIDADES: [&str; 3] = ["s", "min", "h"];

pub struct TelaErgonomia {
    pub modelo: String,
    pub modelo_b: String,
    pub unidade_tempo: String,
    pub trabalho: Trabalho<Estudo>,
    pub trabalho_b: Trabalho<Estudo>,
    pub marcacoes: Vec<Marcacao>,
    pub marcacoes_b: Vec<Marcacao>,
    pub metodo: Option<Metodo>,
    pub arquivo_metodo: String,
    pub erro_metodo: String,
}

impl Default for TelaErgonomia {
    fn default() -> Self {
        TelaErgonomia {
            modelo: String::new(),
            modelo_b: String::new(),
            unidade_tempo: "s".into(),
            trabalho: Trabalho::default(),
            trabalho_b: Trabalho::default(),
            marcacoes: Vec::new(),
            marcacoes_b: Vec::new(),
            metodo: None,
            arquivo_metodo: String::new(),
            erro_metodo: String::new(),
        }
    }
}

fn id_marcacao(m: &Marcacao, modelo: &str) -> String {
    format!("{}|{}|{}", m.ator, m.elemento, modelo)
}

/// Marcações gravadas na unidade para um modelo.
pub fn carregar_marcacoes(u: &Unidade, modelo: &str) -> Vec<Marcacao> {
    let fim = format!("|{modelo}");
    u.estado()
        .brutos(MARCACOES)
        .filter(|(k, _)| k.ends_with(&fim))
        .filter_map(|(_, v)| serde_json::from_value(v.clone()).ok())
        .collect()
}

fn combo(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    v: &mut u8,
    nomes: &[&str],
) -> bool {
    let antes = *v;
    egui::ComboBox::from_id_salt(id)
        .width(90.0)
        .selected_text(nomes.get(usize::from(*v)).copied().unwrap_or("?"))
        .show_ui(ui, |ui| {
            for (i, n) in nomes.iter().enumerate() {
                ui.selectable_value(v, i as u8, *n);
            }
        });
    *v != antes
}

fn estudo_em_fio(
    trabalho: &mut Trabalho<Estudo>,
    s: &mut Sessao,
    caminho: &str,
    unidade: String,
) -> bool {
    let Some(u) = s.unidade.as_ref() else {
        return false;
    };
    let modelo = match util::carregar_modelo(u, caminho) {
        Ok(m) => m,
        Err(e) => {
            s.falhar(e);
            return false;
        }
    };
    s.avisar("Rodando o estudo de tempos...");
    trabalho.iniciar(move || {
        let r = capturar(&modelo, 0)?;
        estudar(
            &modelo,
            &r,
            &Opcoes {
                unidade,
                ..Opcoes::default()
            },
        )
    });
    true
}

impl TelaErgonomia {
    /// Método da unidade (o da empresa gravado, ou o padrão do projeto).
    pub fn metodo_atual(&mut self, s: &Sessao) -> Metodo {
        if self.metodo.is_none() {
            let m = s
                .unidade
                .as_ref()
                .and_then(|u| u.estado().um::<Metodo>(CONFIG, ID_METODO))
                .unwrap_or_default();
            self.metodo = Some(m);
        }
        self.metodo.clone().unwrap_or_default()
    }

    pub fn rodar(&mut self, s: &mut Sessao) {
        let (c, un) = (self.modelo.clone(), self.unidade_tempo.clone());
        estudo_em_fio(&mut self.trabalho, s, &c, un);
    }

    pub fn rodar_b(&mut self, s: &mut Sessao) {
        let (c, un) = (self.modelo_b.clone(), self.unidade_tempo.clone());
        estudo_em_fio(&mut self.trabalho_b, s, &c, un);
    }

    /// Grava ou troca a marcação de um elemento (só na memória, até "Gravar").
    pub fn marcar(&mut self, m: Marcacao) {
        match self
            .marcacoes
            .iter_mut()
            .find(|x| x.ator == m.ator && x.elemento == m.elemento)
        {
            Some(x) => *x = m,
            None => self.marcacoes.push(m),
        }
    }

    /// Notas dos postos do estudo, com as marcações atuais e o método da unidade.
    pub fn notas(&mut self, s: &Sessao) -> Vec<NotaPosto> {
        let metodo = self.metodo_atual(s);
        match self.trabalho.ok() {
            Some(e) => avaliar(e, &self.marcacoes, &metodo),
            None => Vec::new(),
        }
    }

    /// Grava as marcações do modelo estudado na unidade.
    pub fn gravar_marcacoes(&mut self, s: &mut Sessao) -> usize {
        let Some(modelo) = self.trabalho.ok().map(|e| e.modelo.clone()) else {
            return 0;
        };
        let mut n = 0;
        if let Some(u) = s.unidade.as_mut() {
            for m in &self.marcacoes {
                if u.gravar(MARCACOES, &id_marcacao(m, &modelo), m).is_ok() {
                    n += 1;
                }
            }
        }
        s.mudou();
        n
    }

    pub fn importar_metodo(&mut self, s: &mut Sessao) {
        self.erro_metodo.clear();
        let texto = match std::fs::read_to_string(self.arquivo_metodo.trim()) {
            Ok(t) => t,
            Err(e) => {
                self.erro_metodo = e.to_string();
                return;
            }
        };
        match Metodo::de_csv(&texto) {
            Ok(m) => {
                if let Some(u) = s.unidade.as_mut()
                    && let Err(e) = u.gravar(CONFIG, ID_METODO, &m)
                {
                    self.erro_metodo = e.to_string();
                    return;
                }
                s.mudou();
                s.avisar(format!("Método '{}' gravado na unidade.", m.nome));
                self.metodo = Some(m);
            }
            Err(e) => self.erro_metodo = e,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        if s.unidade.is_none() {
            ui.label("Abra uma unidade na tela inicial.");
            return;
        }
        ui.heading("Ergonomia");
        ui.colored_label(egui::Color32::from_rgb(230, 160, 40), AVISO_NR17);
        if self.trabalho.sondar(ui.ctx()) {
            match self.trabalho.erro() {
                Some(e) => s.falhar(e.to_string()),
                None => {
                    if let (Some(u), Some(e)) = (s.unidade.as_ref(), self.trabalho.ok()) {
                        self.marcacoes = carregar_marcacoes(u, &e.modelo);
                    }
                    s.avisar("Estudo de tempos pronto.");
                }
            }
        }
        if self.trabalho_b.sondar(ui.ctx()) {
            match self.trabalho_b.erro() {
                Some(e) => s.falhar(e.to_string()),
                None => {
                    if let (Some(u), Some(e)) = (s.unidade.as_ref(), self.trabalho_b.ok()) {
                        self.marcacoes_b = carregar_marcacoes(u, &e.modelo);
                    }
                    s.avisar("Estudo do segundo modelo pronto.");
                }
            }
        }
        let metodo = self.metodo_atual(s);
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
            utsim_posto_ui::util::campo_modelo(ui, &mut self.modelo, s.unidade.as_ref(), 380.0);
            if !modelos.is_empty() {
                egui::ComboBox::from_id_salt("modelos_ergonomia")
                    .selected_text("modelos dos postos")
                    .show_ui(ui, |ui| {
                        for m in &modelos {
                            ui.selectable_value(&mut self.modelo, m.clone(), m);
                        }
                    });
            }
            ui.label("Unidade de tempo:");
            egui::ComboBox::from_id_salt("unidade_tempo_ergonomia")
                .selected_text(self.unidade_tempo.clone())
                .show_ui(ui, |ui| {
                    for u in UNIDADES {
                        ui.selectable_value(&mut self.unidade_tempo, u.to_string(), u);
                    }
                });
            let livre = !self.trabalho.ocupado() && !self.modelo.trim().is_empty();
            if ui
                .add_enabled(livre, egui::Button::new("Rodar estudo"))
                .clicked()
            {
                self.rodar(s);
            }
            if self.trabalho.ocupado() {
                ui.spinner();
            }
        });
        ui.horizontal(|ui| {
            ui.label(format!("Método: {}", metodo.nome));
            ui.label("Importar método (CSV):");
            utsim_ui_tema::arquivos::campo(
                ui,
                &mut self.arquivo_metodo,
                utsim_ui_tema::arquivos::Tipo::Abrir,
                "Escolha o arquivo CSV",
                &[utsim_ui_tema::arquivos::CSV],
                300.0,
            );
            if ui.button("Importar").clicked() {
                self.importar_metodo(s);
            }
        });
        if !self.erro_metodo.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(230, 90, 80), &self.erro_metodo);
        }
        let Some(estudo) = self.trabalho.ok() else {
            if let Some(e) = self.trabalho.erro() {
                ui.colored_label(egui::Color32::from_rgb(230, 90, 80), e);
            }
            return;
        };
        let (nome_modelo, unidade) = (estudo.modelo.clone(), estudo.unidade.clone());
        ui.separator();
        ui.strong(format!(
            "Modelo {nome_modelo} · tempos em {unidade} · Origem: Simulado"
        ));
        let notas = avaliar(estudo, &self.marcacoes, &metodo);
        if notas.is_empty() {
            ui.label("O estudo não achou operador com ciclo regular neste modelo.");
        }
        let mut mudancas: Vec<Marcacao> = Vec::new();
        for p in &notas {
            ui.add_space(6.0);
            ui.strong(format!(
                "{}: nota {:.1} ({}), média ponderada {:.1}",
                p.ator, p.nota, p.faixa, p.media_ponderada
            ));
            egui::Grid::new(("ergo", &p.ator))
                .striped(true)
                .show(ui, |ui| {
                    for c in [
                        "Elemento",
                        "Por hora",
                        "Tronco",
                        "Braços",
                        "Pescoço",
                        "Pernas",
                        "Carga (kg)",
                        "Pega",
                        "Vibração",
                        "Nota",
                        "Faixa",
                    ] {
                        ui.strong(c);
                    }
                    ui.end_row();
                    for e in &p.elementos {
                        let atual = self
                            .marcacoes
                            .iter()
                            .find(|m| m.ator == e.ator && m.elemento == e.elemento);
                        let mut m = atual.cloned().unwrap_or_else(|| Marcacao {
                            ator: e.ator.clone(),
                            elemento: e.elemento.clone(),
                            ..Marcacao::default()
                        });
                        let id = (&e.ator, &e.elemento);
                        ui.label(&e.elemento);
                        ui.label(format!("{:.0}", e.por_hora));
                        let mut mudou = false;
                        mudou |= combo(ui, (id, "tronco"), &mut m.tronco, &POSTURA);
                        mudou |= combo(ui, (id, "bracos"), &mut m.bracos, &POSTURA);
                        mudou |= combo(ui, (id, "pescoco"), &mut m.pescoco, &POSTURA);
                        mudou |= combo(ui, (id, "pernas"), &mut m.pernas, &POSTURA);
                        mudou |= ui
                            .add(
                                egui::DragValue::new(&mut m.carga_kg)
                                    .range(0.0..=200.0)
                                    .speed(0.5),
                            )
                            .changed();
                        mudou |= combo(ui, (id, "pega"), &mut m.pega, &PEGA);
                        mudou |= combo(ui, (id, "vibracao"), &mut m.vibracao, &VIBRACAO);
                        ui.label(format!("{:.1}", e.nota));
                        ui.label(e.faixa);
                        ui.end_row();
                        if mudou {
                            mudancas.push(m);
                        }
                    }
                });
        }
        for m in mudancas {
            self.marcar(m);
        }
        ui.add_space(6.0);
        let mut gravar = false;
        let mut gravar_html = false;
        ui.horizontal(|ui| {
            gravar = ui.button("Gravar marcações").clicked();
            gravar_html = ui.button("Gravar HTML de ergonomia").clicked();
        });
        if gravar {
            let n = self.gravar_marcacoes(s);
            s.avisar(format!("{n} marcações gravadas na unidade."));
        }
        // Comparação com um segundo modelo.
        ui.separator();
        ui.strong("Comparar com outro modelo");
        ui.horizontal(|ui| {
            ui.label("Segundo modelo (.ron):");
            utsim_posto_ui::util::campo_modelo(ui, &mut self.modelo_b, s.unidade.as_ref(), 380.0);
            if !modelos.is_empty() {
                egui::ComboBox::from_id_salt("modelos_ergonomia_b")
                    .selected_text("modelos dos postos")
                    .show_ui(ui, |ui| {
                        for m in &modelos {
                            ui.selectable_value(&mut self.modelo_b, m.clone(), m);
                        }
                    });
            }
            let livre = !self.trabalho_b.ocupado() && !self.modelo_b.trim().is_empty();
            if ui
                .add_enabled(livre, egui::Button::new("Rodar e comparar"))
                .clicked()
            {
                self.rodar_b(s);
            }
            if self.trabalho_b.ocupado() {
                ui.spinner();
            }
        });
        let comparacao = match self.trabalho_b.ok() {
            Some(b) => comparar(&notas, &avaliar(b, &self.marcacoes_b, &metodo)),
            None => Vec::new(),
        };
        if let Some(e) = self.trabalho_b.erro() {
            ui.colored_label(egui::Color32::from_rgb(230, 90, 80), e);
        }
        if !comparacao.is_empty() {
            egui::Grid::new("ergo_comparacao")
                .striped(true)
                .show(ui, |ui| {
                    for c in [
                        "Posto",
                        "Nota antes",
                        "Nota depois",
                        "Padrão antes",
                        "Padrão depois",
                    ] {
                        ui.strong(c);
                    }
                    ui.end_row();
                    let opt = |v: Option<f64>| v.map_or("–".to_string(), |x| format!("{x:.2}"));
                    for c in &comparacao {
                        ui.label(&c.ator);
                        ui.label(format!("{:.1}", c.nota_antes));
                        ui.label(format!("{:.1}", c.nota_depois));
                        ui.label(opt(c.padrao_antes));
                        ui.label(opt(c.padrao_depois));
                        ui.end_row();
                    }
                });
        }
        if gravar_html {
            let doc = html(&nome_modelo, &metodo, &notas, &comparacao);
            if let Some(u) = s.unidade.as_ref() {
                match u.gravar_relatorio("ergonomia.html", doc.as_bytes()) {
                    Ok(p) => s.avisar(format!("Relatório gravado em {}", p.display())),
                    Err(e) => s.falhar(e.to_string()),
                }
            }
        }
    }
}
