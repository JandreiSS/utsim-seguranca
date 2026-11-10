//! Tela de incidentes: registro, horas-homem, indicadores do período e a sub-tela das
//! pessoas da pasta restrita. Pessoas só aparecem pelo código. O nome só é mostrado no
//! registro, e só quando a pasta restrita abre; os indicadores nunca mostram nomes.

use eframe::egui::{self, Color32};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use utsim_posto::{Pessoa, Restrita};
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::incidentes::{
    CONFIG_INDICADORES, Config, HORAS, HorasHomem, INCIDENTES, Incidente, Indicadores, Tipo,
    carta_mensal, indicadores,
};

const VERMELHO: Color32 = Color32::from_rgb(230, 90, 80);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aba {
    #[default]
    Registro,
    Horas,
    Indicadores,
    Pessoas,
}

#[derive(Default)]
pub struct TelaIncidentes {
    pub aba: Aba,
    pub novo: Option<Incidente>,
    pub nova_causa: String,
    pub horas: HorasHomem,
    /// Período dos indicadores (`AAAA-MM-DD`); vazio: do começo do ano até hoje.
    pub de: String,
    pub ate: String,
    /// Base das taxas digitada (absente: a gravada ou 1 000 000).
    pub base: Option<f64>,
    /// Pasta restrita aberta e a versão da unidade com que foi aberta.
    restrita: Option<Restrita>,
    restrita_versao: Option<u64>,
    /// Pasta restrita fora da unidade (vazio: dentro dela) e quem pode gravar.
    pub caminho_restrita: String,
    pub escritores: String,
    pub pessoa: Pessoa,
    pub confirmar: Option<String>,
}

fn campo(ui: &mut egui::Ui, rotulo: &str, texto: &mut String) {
    ui.horizontal(|ui| {
        ui.label(rotulo);
        ui.add(egui::TextEdit::singleline(texto).desired_width(f32::INFINITY));
    });
}

impl TelaIncidentes {
    pub fn incidentes(s: &Sessao) -> Vec<Incidente> {
        let mut v: Vec<Incidente> = s
            .unidade
            .as_ref()
            .map(|u| u.estado().lista(INCIDENTES))
            .unwrap_or_default();
        v.sort_by(|a, b| b.data.cmp(&a.data).then(a.id.cmp(&b.id)));
        v
    }

    pub fn horas_homem(s: &Sessao) -> Vec<HorasHomem> {
        let mut v: Vec<HorasHomem> = s
            .unidade
            .as_ref()
            .map(|u| u.estado().lista(HORAS))
            .unwrap_or_default();
        v.sort_by(|a, b| a.mes.cmp(&b.mes));
        v
    }

    /// A pasta restrita, aberta de novo quando a unidade mudou. `None`: ela não abre para
    /// esta pessoa (ou não existe).
    pub fn restrita(&mut self, s: &Sessao) -> Option<&mut Restrita> {
        if self.restrita_versao != Some(s.versao) {
            self.restrita = s.unidade.as_ref().and_then(|u| u.abrir_restrita());
            self.restrita_versao = Some(s.versao);
        }
        self.restrita.as_mut()
    }

    /// Descarta a pasta restrita guardada, para reabrir no próximo quadro.
    pub fn reler_restrita(&mut self) {
        self.restrita_versao = None;
    }

    /// Texto de uma pessoa no registro: o código e, se a pasta restrita abre, o nome.
    fn rotulo(pessoas: &[Pessoa], aberta: bool, codigo: &str) -> String {
        match pessoas.iter().find(|p| p.codigo == codigo) {
            Some(p) if aberta => format!("{codigo} ({})", p.nome),
            _ => codigo.to_string(),
        }
    }

    /// Grava o incidente em edição; recusa quando `problemas()` não está vazio.
    pub fn gravar(&mut self, s: &mut Sessao) -> bool {
        let Some(i) = self.novo.as_ref() else {
            return false;
        };
        let problemas = i.problemas();
        if !problemas.is_empty() {
            s.falhar(format!("Incidente não gravado: {}.", problemas.join("; ")));
            return false;
        }
        if i.descricao.trim().is_empty() {
            s.falhar("Incidente não gravado: descreva o que aconteceu.");
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(INCIDENTES, &i.id.clone(), i);
        if s.resultado(r, "Incidente gravado.").is_some() {
            self.novo = None;
            true
        } else {
            false
        }
    }

    /// Grava as horas-homem de um mês (um registro por mês).
    pub fn gravar_horas(&mut self, s: &mut Sessao) -> bool {
        let h = self.horas.clone();
        if h.mes.len() != 7 || !h.horas.is_finite() || h.horas <= 0.0 {
            s.falhar("Informe o mês (AAAA-MM) e as horas.");
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(HORAS, &h.mes, &h);
        s.resultado(r, "Horas-homem gravadas.").is_some()
    }

    pub fn config(&self, s: &Sessao) -> Config {
        let gravada: Config = s
            .unidade
            .as_ref()
            .and_then(|u| u.estado().um(CONFIG_INDICADORES, "base"))
            .unwrap_or_default();
        Config {
            base: self.base.unwrap_or(gravada.base),
        }
    }

    pub fn gravar_base(&mut self, s: &mut Sessao) -> bool {
        let c = self.config(s);
        if !(c.base.is_finite() && c.base > 0.0) {
            s.falhar("A base das taxas deve ser positiva.");
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(CONFIG_INDICADORES, "base", &c);
        s.resultado(r, "Base das taxas gravada.").is_some()
    }

    /// Indicadores do período escolhido.
    pub fn calcular(&self, s: &Sessao) -> Indicadores {
        let hoje = util::hoje();
        let de = if self.de.is_empty() {
            format!("{}-01-01", &hoje[..4])
        } else {
            self.de.clone()
        };
        let ate = if self.ate.is_empty() {
            hoje.clone()
        } else {
            self.ate.clone()
        };
        indicadores(
            &Self::incidentes(s),
            &Self::horas_homem(s),
            self.config(s),
            &de,
            &ate,
            &hoje,
        )
    }

    /// Cria a pasta restrita da unidade.
    pub fn criar_restrita(&mut self, s: &mut Sessao) -> bool {
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let mut escritores: Vec<String> = self
            .escritores
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect();
        if escritores.is_empty() {
            escritores.push(u.autor().to_string());
        }
        let caminho = self.caminho_restrita.trim().to_string();
        let r = u.criar_restrita(
            (!caminho.is_empty()).then(|| std::path::Path::new(&caminho)),
            &escritores,
        );
        match r {
            Ok(r) => {
                self.restrita = Some(r);
                self.restrita_versao = Some(s.versao);
                s.avisar("Pasta restrita criada.");
                true
            }
            Err(e) => {
                s.falhar(e.to_string());
                false
            }
        }
    }

    /// Cadastra a pessoa em edição na pasta restrita. Devolve o código.
    pub fn cadastrar(&mut self, s: &mut Sessao) -> Option<String> {
        let p = self.pessoa.clone();
        if p.nome.trim().len() < 2 {
            s.falhar("Informe o nome da pessoa.");
            return None;
        }
        let r = self.restrita(s)?.cadastrar(p);
        match r {
            Ok(c) => {
                self.pessoa = Pessoa::default();
                s.avisar(format!("Pessoa cadastrada com o código {c}."));
                Some(c)
            }
            Err(e) => {
                s.falhar(e.to_string());
                None
            }
        }
    }

    pub fn anonimizar(&mut self, s: &mut Sessao, codigo: &str) -> bool {
        let Some(r) = self.restrita(s) else {
            return false;
        };
        match r.anonimizar(codigo) {
            Ok(()) => {
                s.avisar(format!("{codigo} anonimizada."));
                true
            }
            Err(e) => {
                s.falhar(e.to_string());
                false
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        if s.unidade.is_none() {
            ui.label("Abra uma unidade na tela inicial.");
            return;
        }
        ui.heading("Incidentes");
        ui.horizontal(|ui| {
            for (a, nome) in [
                (Aba::Registro, "Registro"),
                (Aba::Horas, "Horas-homem"),
                (Aba::Indicadores, "Indicadores"),
                (Aba::Pessoas, "Pessoas (restrita)"),
            ] {
                if ui.selectable_label(self.aba == a, nome).clicked() {
                    self.aba = a;
                }
            }
        });
        ui.separator();
        match self.aba {
            Aba::Registro => self.registro_ui(ui, s),
            Aba::Horas => self.horas_ui(ui, s),
            Aba::Indicadores => self.indicadores_ui(ui, s),
            Aba::Pessoas => self.pessoas_ui(ui, s),
        }
    }

    fn registro_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        let postos = s.unidade.as_ref().map(|u| u.postos()).unwrap_or_default();
        let (pessoas, aberta) = match self.restrita(s) {
            Some(r) => (r.pessoas(), true),
            None => (Vec::new(), false),
        };
        if !aberta {
            ui.weak("Pasta restrita indisponível: as pessoas aparecem só pelo código.");
        }
        let lista = Self::incidentes(s);
        if lista.is_empty() {
            ui.weak("Nenhum incidente registrado.");
        } else {
            let mut editar = None;
            egui::Grid::new("grade_incidentes")
                .striped(true)
                .show(ui, |ui| {
                    for c in ["Data", "Posto", "Tipo", "Descrição", "Pessoas", ""] {
                        ui.strong(c);
                    }
                    ui.end_row();
                    for i in &lista {
                        ui.label(&i.data);
                        ui.label(
                            postos
                                .iter()
                                .find(|p| p.id == i.posto)
                                .map_or(i.posto.as_str(), |p| p.nome.as_str()),
                        );
                        ui.label(i.tipo.nome());
                        ui.label(&i.descricao);
                        let nomes: Vec<String> = i
                            .pessoas
                            .iter()
                            .map(|c| Self::rotulo(&pessoas, aberta, c))
                            .collect();
                        ui.label(nomes.join(", "));
                        if ui.small_button("Editar").clicked() {
                            editar = Some(i.clone());
                        }
                        ui.end_row();
                    }
                });
            if let Some(i) = editar {
                self.novo = Some(i);
            }
        }
        if ui.button("Novo incidente").clicked() {
            self.novo = Some(Incidente {
                data: util::hoje(),
                ..Incidente::default()
            });
        }
        let (mut gravar, mut cancelar) = (false, false);
        let nova_causa = &mut self.nova_causa;
        if let Some(i) = self.novo.as_mut() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Data (AAAA-MM-DD)");
                    ui.add(egui::TextEdit::singleline(&mut i.data).desired_width(100.0));
                    ui.label("Hora");
                    ui.add(egui::TextEdit::singleline(&mut i.hora).desired_width(60.0));
                    ui.label("Tipo");
                    egui::ComboBox::from_id_salt("tipo_incidente")
                        .selected_text(i.tipo.nome())
                        .show_ui(ui, |ui| {
                            for t in Tipo::TODOS {
                                ui.selectable_value(&mut i.tipo, t, t.nome());
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Posto");
                    let nome = postos
                        .iter()
                        .find(|p| p.id == i.posto)
                        .map_or("(nenhum)", |p| p.nome.as_str());
                    egui::ComboBox::from_id_salt("posto_incidente")
                        .selected_text(nome)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut i.posto, String::new(), "(nenhum)");
                            for p in &postos {
                                ui.selectable_value(&mut i.posto, p.id.clone(), &p.nome);
                            }
                        });
                    ui.label("Zona do layout");
                    ui.add(egui::TextEdit::singleline(&mut i.zona).desired_width(120.0));
                });
                campo(ui, "O que aconteceu", &mut i.descricao);
                campo(ui, "Ações tomadas", &mut i.acoes);
                let mut tira = None;
                for (k, c) in i.causas.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Causa: {c}"));
                        if ui.small_button("tirar").clicked() {
                            tira = Some(k);
                        }
                    });
                }
                if let Some(k) = tira {
                    i.causas.remove(k);
                }
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(nova_causa)
                            .hint_text("causa")
                            .desired_width(240.0),
                    );
                    if ui.button("Incluir causa").clicked() && !nova_causa.trim().is_empty() {
                        i.causas.push(nova_causa.trim().to_string());
                        nova_causa.clear();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Dias perdidos");
                    ui.add(egui::DragValue::new(&mut i.dias_perdidos).range(0.0..=10_000.0));
                    ui.label("Dias debitados (tabela da empresa)");
                    ui.add(egui::DragValue::new(&mut i.dias_debitados).range(0.0..=100_000.0));
                });
                ui.label("Pessoas envolvidas (da pasta restrita, pelo código):");
                if pessoas.is_empty() {
                    ui.weak("Nenhuma pessoa disponível.");
                }
                for p in pessoas.iter().filter(|p| !p.anonimizada) {
                    let mut marcada = i.pessoas.contains(&p.codigo);
                    if ui
                        .checkbox(&mut marcada, format!("{} ({})", p.codigo, p.nome))
                        .changed()
                    {
                        i.pessoas.retain(|c| c != &p.codigo);
                        if marcada {
                            i.pessoas.push(p.codigo.clone());
                        }
                    }
                }
                for c in i
                    .pessoas
                    .iter()
                    .filter(|c| !pessoas.iter().any(|p| &p.codigo == *c))
                {
                    ui.colored_label(VERMELHO, format!("'{c}' não está na pasta restrita"));
                }
                ui.horizontal(|ui| {
                    gravar = ui.button("Gravar incidente").clicked();
                    cancelar = ui.button("Cancelar").clicked();
                });
            });
        }
        if gravar {
            self.gravar(s);
        }
        if cancelar {
            self.novo = None;
        }
    }

    fn horas_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        ui.label("Horas de exposição por mês, a base das taxas.");
        egui::Grid::new("grade_horas").striped(true).show(ui, |ui| {
            ui.strong("Mês");
            ui.strong("Horas");
            ui.end_row();
            for h in Self::horas_homem(s) {
                ui.label(&h.mes);
                ui.label(format!("{:.1}", h.horas));
                ui.end_row();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Mês (AAAA-MM)");
            ui.add(egui::TextEdit::singleline(&mut self.horas.mes).desired_width(80.0));
            ui.label("Horas");
            ui.add(egui::DragValue::new(&mut self.horas.horas).speed(10.0));
            if ui.button("Gravar horas").clicked() {
                self.gravar_horas(s);
            }
        });
    }

    fn indicadores_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        ui.horizontal(|ui| {
            ui.label("De (AAAA-MM-DD)");
            ui.add(egui::TextEdit::singleline(&mut self.de).desired_width(100.0));
            ui.label("Até");
            ui.add(egui::TextEdit::singleline(&mut self.ate).desired_width(100.0));
            ui.label("Taxas por");
            let mut b = self.config(s).base;
            if ui
                .add(egui::DragValue::new(&mut b).speed(1000.0).suffix(" h"))
                .changed()
            {
                self.base = Some(b);
            }
            if ui.button("Gravar base").clicked() {
                self.gravar_base(s);
            }
        });
        let ind = self.calcular(s);
        let config = self.config(s);
        ui.add_space(4.0);
        egui::Grid::new("grade_indicadores")
            .striped(true)
            .show(ui, |ui| {
                let linha = |ui: &mut egui::Ui, n: &str, v: String| {
                    ui.label(n);
                    ui.label(v);
                    ui.end_row();
                };
                linha(ui, "Horas de exposição", format!("{:.1}", ind.horas));
                for t in Tipo::TODOS {
                    linha(
                        ui,
                        t.nome(),
                        ind.por_tipo.get(&t).copied().unwrap_or(0).to_string(),
                    );
                }
                linha(
                    ui,
                    "Taxa de frequência",
                    format!("{:.2}", ind.taxa_frequencia),
                );
                linha(
                    ui,
                    "Taxa de frequência com afastamento",
                    format!("{:.2}", ind.taxa_frequencia_afastamento),
                );
                linha(
                    ui,
                    "Taxa de gravidade",
                    format!("{:.2}", ind.taxa_gravidade),
                );
                linha(
                    ui,
                    "Quase-acidentes por lesão",
                    ind.razao_quase.map_or("–".into(), |r| format!("{r:.2}")),
                );
                linha(
                    ui,
                    "Dias sem acidente com lesão",
                    ind.dias_sem_acidente.map_or("–".into(), |d| d.to_string()),
                );
            });
        if ind.horas <= 0.0 {
            ui.colored_label(
                VERMELHO,
                "Sem horas-homem no período: as taxas ficam em zero.",
            );
        }
        for (titulo, p, id) in [
            ("Pareto das causas", &ind.pareto_causas, "pareto_causas"),
            ("Pareto dos postos", &ind.pareto_postos, "pareto_postos"),
        ] {
            ui.add_space(6.0);
            ui.strong(titulo);
            if p.rows.is_empty() {
                ui.weak("Sem dados no período.");
                continue;
            }
            let nomes: Vec<String> = p.rows.iter().map(|r| r.name.clone()).collect();
            let barras: Vec<Bar> = p
                .rows
                .iter()
                .enumerate()
                .map(|(i, r)| Bar::new(i as f64, r.value).width(0.7).name(&r.name))
                .collect();
            let n = nomes.len();
            Plot::new(id)
                .height(180.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .include_y(0.0)
                .x_axis_formatter(move |marca, _| {
                    let k = marca.value.round();
                    if (marca.value - k).abs() < 1e-6 && k >= 0.0 && (k as usize) < n {
                        nomes[k as usize].clone()
                    } else {
                        String::new()
                    }
                })
                .show(ui, |pl| pl.bar_chart(BarChart::new("eventos", barras)));
        }
        if !ind.por_zona.is_empty() {
            ui.add_space(6.0);
            ui.strong("Eventos por zona do layout");
            for (z, n) in &ind.por_zona {
                ui.label(format!("{z}: {n}"));
            }
        }
        ui.add_space(6.0);
        ui.strong("Lesões por mês (carta u)");
        match carta_mensal(&ind, config) {
            None => {
                ui.weak("A carta precisa de ao menos dois meses com horas-homem.");
            }
            Some(ch) => {
                ui.label(if ch.in_control() {
                    "Sem sinais na carta."
                } else {
                    "Há sinais na carta: o mês fora do padrão merece investigação."
                });
                for (k, p) in ch.panels.iter().enumerate() {
                    let linha = |v: &[f64]| -> PlotPoints {
                        v.iter()
                            .enumerate()
                            .map(|(i, y)| [(i + 1) as f64, *y])
                            .collect()
                    };
                    let valores: PlotPoints = p
                        .values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| v.map(|v| [(i + 1) as f64, v]))
                        .collect();
                    Plot::new(format!("carta_u_{k}"))
                        .height(150.0)
                        .allow_scroll(false)
                        .show(ui, |pl| {
                            pl.line(Line::new("taxa", valores));
                            pl.line(Line::new("central", linha(&p.center)));
                            pl.line(Line::new("LSC", linha(&p.ucl)));
                            pl.line(Line::new("LIC", linha(&p.lcl)));
                        });
                }
            }
        }
    }

    fn pessoas_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        ui.label(
            "Dados pessoais ficam só na pasta restrita (LGPD). No resto da unidade a pessoa \
             aparece pelo código.",
        );
        let aberta = self.restrita(s).is_some();
        if !aberta {
            ui.colored_label(
                VERMELHO,
                "A pasta restrita não existe ou não abre para você.",
            );
            ui.horizontal(|ui| {
                ui.label("Pasta fora da unidade (vazio: dentro da unidade)");
                utsim_ui_tema::arquivos::campo(
                    ui,
                    &mut self.caminho_restrita,
                    utsim_ui_tema::arquivos::Tipo::Pasta,
                    "Escolha a pasta restrita",
                    &[],
                    360.0,
                );
            });
            campo(
                ui,
                "Quem pode gravar (autores separados por vírgula; vazio: só você)",
                &mut self.escritores,
            );
            if ui.button("Criar pasta restrita").clicked() {
                self.criar_restrita(s);
            }
            return;
        }
        let (pessoas, pode) = match self.restrita(s) {
            Some(r) => (r.pessoas(), r.pode_escrever()),
            None => return,
        };
        if ui.button("Reler a pasta restrita").clicked() {
            self.reler_restrita();
        }
        egui::Grid::new("grade_pessoas")
            .striped(true)
            .show(ui, |ui| {
                for c in ["Código", "Nome", "Setor", "Consentimento", ""] {
                    ui.strong(c);
                }
                ui.end_row();
                for p in &pessoas {
                    ui.label(&p.codigo);
                    ui.label(&p.nome);
                    ui.label(&p.setor);
                    ui.label(if p.consentimento { "sim" } else { "não" });
                    if pode && !p.anonimizada {
                        if self.confirmar.as_deref() == Some(p.codigo.as_str()) {
                            ui.horizontal(|ui| {
                                if ui
                                    .small_button("Confirmar: apaga o nome para sempre")
                                    .clicked()
                                {
                                    let c = p.codigo.clone();
                                    self.confirmar = None;
                                    self.anonimizar(s, &c);
                                }
                                if ui.small_button("Cancelar").clicked() {
                                    self.confirmar = None;
                                }
                            });
                        } else if ui.small_button("Anonimizar").clicked() {
                            self.confirmar = Some(p.codigo.clone());
                        }
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
        if !pode {
            ui.colored_label(
                VERMELHO,
                "Você não está entre as pessoas que podem gravar dados pessoais nesta unidade.",
            );
            return;
        }
        ui.separator();
        ui.strong("Cadastrar pessoa");
        campo(ui, "Nome", &mut self.pessoa.nome);
        campo(ui, "Setor", &mut self.pessoa.setor);
        ui.checkbox(&mut self.pessoa.consentimento, "Consentimento registrado");
        if ui.button("Cadastrar").clicked() {
            self.cadastrar(s);
        }
    }
}
