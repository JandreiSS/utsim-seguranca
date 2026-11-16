//! Tela de incidentes: registro, horas-homem, indicadores do período e a sub-tela das
//! pessoas da pasta restrita. Pessoas só aparecem pelo código. O nome só é mostrado no
//! registro, e só quando a pasta restrita abre; os indicadores nunca mostram nomes.

use crate::texto::{fx, tx, vx};
use eframe::egui::{self, Color32};
use egui_plot::{Bar, BarChart, Plot};
use utsim_posto::{Pessoa, Restrita};
use utsim_posto_ui::{Sessao, util};
use utsim_seguranca_core::incidentes::{
    CONFIG_INDICADORES, Config, HORAS, HorasHomem, INCIDENTES, Incidente, Indicadores, Tipo,
    carta_mensal, indicadores,
};
use utsim_ui_tema::campos::texto_com_rotulo as campo;

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
            s.falhar(fx(
                "incidentes-incidente_nao_gravado",
                &[("problemas", vx(&problemas.join("; ")))],
            ));
            return false;
        }
        if i.descricao.trim().is_empty() {
            s.falhar(tx("incidentes-incidente_nao_gravado_descreva_o"));
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(INCIDENTES, &i.id.clone(), i);
        if s.resultado(r, tx("incidentes-incidente_gravado")).is_some() {
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
            s.falhar(tx("incidentes-informe_o_mes_aaaa_mm"));
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(HORAS, &h.mes, &h);
        s.resultado(r, tx("incidentes-horas_homem_gravadas"))
            .is_some()
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
            s.falhar(tx("incidentes-a_base_das_taxas_deve"));
            return false;
        }
        let Some(u) = s.unidade.as_mut() else {
            return false;
        };
        let r = u.gravar(CONFIG_INDICADORES, "base", &c);
        s.resultado(r, tx("incidentes-base_das_taxas_gravada"))
            .is_some()
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
                s.avisar(tx("incidentes-pasta_restrita_criada"));
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
            s.falhar(tx("incidentes-informe_o_nome_da_pessoa"));
            return None;
        }
        let r = self.restrita(s)?.cadastrar(p);
        match r {
            Ok(c) => {
                self.pessoa = Pessoa::default();
                s.avisar(fx(
                    "incidentes-pessoa_cadastrada_com_o_codigo",
                    &[("c", vx(&c))],
                ));
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
                s.avisar(fx("incidentes-anonimizada", &[("codigo", vx(&codigo))]));
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
            ui.label(tx("incidentes-abra_uma_unidade_na_tela"));
            return;
        }
        ui.heading(tx("incidentes-incidentes"));
        ui.horizontal(|ui| {
            for (a, nome) in [
                (Aba::Registro, tx("incidentes-registro")),
                (Aba::Horas, tx("incidentes-horas_homem")),
                (Aba::Indicadores, tx("incidentes-indicadores")),
                (Aba::Pessoas, tx("incidentes-pessoas_restrita")),
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
            ui.weak(tx("incidentes-pasta_restrita_indisponivel_as_pessoas"));
        }
        let lista = Self::incidentes(s);
        if lista.is_empty() {
            ui.weak(tx("incidentes-nenhum_incidente_registrado"));
        } else {
            let mut editar = None;
            egui::Grid::new("grade_incidentes")
                .striped(true)
                .show(ui, |ui| {
                    for c in [
                        tx("incidentes-data"),
                        tx("incidentes-posto"),
                        tx("incidentes-tipo"),
                        tx("incidentes-descricao"),
                        tx("incidentes-pessoas"),
                        "",
                    ] {
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
                        if ui.small_button(tx("incidentes-editar")).clicked() {
                            editar = Some(i.clone());
                        }
                        ui.end_row();
                    }
                });
            if let Some(i) = editar {
                self.novo = Some(i);
            }
        }
        if ui.button(tx("incidentes-novo_incidente")).clicked() {
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
                    ui.label(tx("incidentes-data_aaaa_mm_dd"));
                    util::campo_data(ui, &mut i.data, None, 100.0);
                    ui.label(tx("incidentes-hora"));
                    ui.add(egui::TextEdit::singleline(&mut i.hora).desired_width(60.0));
                    ui.label(tx("incidentes-tipo"));
                    egui::ComboBox::from_id_salt("tipo_incidente")
                        .selected_text(i.tipo.nome())
                        .show_ui(ui, |ui| {
                            for t in Tipo::TODOS {
                                ui.selectable_value(&mut i.tipo, t, t.nome());
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(tx("incidentes-posto"));
                    let nome = postos
                        .iter()
                        .find(|p| p.id == i.posto)
                        .map_or(tx("incidentes-nenhum"), |p| p.nome.as_str());
                    egui::ComboBox::from_id_salt("posto_incidente")
                        .selected_text(nome)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut i.posto,
                                String::new(),
                                tx("incidentes-nenhum"),
                            );
                            for p in &postos {
                                ui.selectable_value(&mut i.posto, p.id.clone(), &p.nome);
                            }
                        });
                    ui.label(tx("incidentes-zona_do_layout"));
                    ui.add(egui::TextEdit::singleline(&mut i.zona).desired_width(120.0));
                });
                campo(ui, tx("incidentes-o_que_aconteceu"), &mut i.descricao);
                campo(ui, tx("incidentes-acoes_tomadas"), &mut i.acoes);
                let mut tira = None;
                for (k, c) in i.causas.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(fx("incidentes-causa", &[("c", vx(&c))]));
                        if ui.small_button(tx("incidentes-tirar")).clicked() {
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
                            .hint_text(tx("incidentes-causa-2"))
                            .desired_width(240.0),
                    );
                    if ui.button(tx("incidentes-incluir_causa")).clicked()
                        && !nova_causa.trim().is_empty()
                    {
                        i.causas.push(nova_causa.trim().to_string());
                        nova_causa.clear();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(tx("incidentes-dias_perdidos"));
                    ui.add(egui::DragValue::new(&mut i.dias_perdidos).range(0.0..=10_000.0));
                    ui.label(tx("incidentes-dias_debitados_tabela_da_empresa"));
                    ui.add(egui::DragValue::new(&mut i.dias_debitados).range(0.0..=100_000.0));
                });
                ui.label(tx("incidentes-pessoas_envolvidas_da_pasta_restrita"));
                if pessoas.is_empty() {
                    ui.weak(tx("incidentes-nenhuma_pessoa_disponivel"));
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
                    ui.colored_label(
                        VERMELHO,
                        fx("incidentes-nao_esta_na_pasta_restrita", &[("c", vx(&c))]),
                    );
                }
                ui.horizontal(|ui| {
                    gravar = ui.button(tx("incidentes-gravar_incidente")).clicked();
                    cancelar = ui.button(tx("incidentes-cancelar")).clicked();
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
        ui.label(tx("incidentes-horas_de_exposicao_por_mes"));
        egui::Grid::new("grade_horas").striped(true).show(ui, |ui| {
            ui.strong(tx("incidentes-mes"));
            ui.strong(tx("incidentes-horas"));
            ui.end_row();
            for h in Self::horas_homem(s) {
                ui.label(&h.mes);
                ui.label(format!("{:.1}", h.horas));
                ui.end_row();
            }
        });
        ui.horizontal(|ui| {
            ui.label(tx("incidentes-mes_aaaa_mm"));
            ui.add(egui::TextEdit::singleline(&mut self.horas.mes).desired_width(80.0));
            ui.label(tx("incidentes-horas"));
            ui.add(egui::DragValue::new(&mut self.horas.horas).speed(10.0));
            if ui.button(tx("incidentes-gravar_horas")).clicked() {
                self.gravar_horas(s);
            }
        });
    }

    fn indicadores_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        ui.horizontal(|ui| {
            ui.label(tx("incidentes-de_aaaa_mm_dd"));
            util::campo_data(ui, &mut self.de, None, 100.0);
            ui.label(tx("incidentes-ate"));
            util::campo_data(ui, &mut self.ate, Some(self.de.as_str()), 100.0);
            ui.label(tx("incidentes-taxas_por"));
            let mut b = self.config(s).base;
            if ui
                .add(
                    egui::DragValue::new(&mut b)
                        .speed(1000.0)
                        .suffix(tx("incidentes-h")),
                )
                .changed()
            {
                self.base = Some(b);
            }
            if ui.button(tx("incidentes-gravar_base")).clicked() {
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
                linha(
                    ui,
                    tx("incidentes-horas_de_exposicao"),
                    format!("{:.1}", ind.horas),
                );
                for t in Tipo::TODOS {
                    linha(
                        ui,
                        t.nome(),
                        ind.por_tipo.get(&t).copied().unwrap_or(0).to_string(),
                    );
                }
                linha(
                    ui,
                    tx("incidentes-taxa_de_frequencia"),
                    format!("{:.2}", ind.taxa_frequencia),
                );
                linha(
                    ui,
                    tx("incidentes-taxa_de_frequencia_com_afastamento"),
                    format!("{:.2}", ind.taxa_frequencia_afastamento),
                );
                linha(
                    ui,
                    tx("incidentes-taxa_de_gravidade"),
                    format!("{:.2}", ind.taxa_gravidade),
                );
                linha(
                    ui,
                    tx("incidentes-quase_acidentes_por_lesao"),
                    ind.razao_quase.map_or("–".into(), |r| format!("{r:.2}")),
                );
                linha(
                    ui,
                    tx("incidentes-dias_sem_acidente_com_lesao"),
                    ind.dias_sem_acidente.map_or("–".into(), |d| d.to_string()),
                );
            });
        if ind.horas <= 0.0 {
            ui.colored_label(VERMELHO, tx("incidentes-sem_horas_homem_no_periodo"));
        }
        for (titulo, p, id) in [
            (
                tx("incidentes-pareto_das_causas"),
                &ind.pareto_causas,
                "pareto_causas",
            ),
            (
                tx("incidentes-pareto_dos_postos"),
                &ind.pareto_postos,
                "pareto_postos",
            ),
        ] {
            ui.add_space(6.0);
            ui.strong(titulo);
            if p.rows.is_empty() {
                ui.weak(tx("incidentes-sem_dados_no_periodo"));
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
            ui.strong(tx("incidentes-eventos_por_zona_do_layout"));
            for (z, n) in &ind.por_zona {
                ui.label(format!("{z}: {n}"));
            }
        }
        ui.add_space(6.0);
        ui.strong(tx("incidentes-lesoes_por_mes_carta_u"));
        match carta_mensal(&ind, config) {
            None => {
                ui.weak(tx("incidentes-a_carta_precisa_de_ao"));
            }
            Some(ch) => {
                ui.label(if ch.in_control() {
                    tx("incidentes-sem_sinais_na_carta")
                } else {
                    tx("incidentes-ha_sinais_na_carta_o")
                });
                let estilo = utsim_posto_ui::carta::Estilo {
                    altura: 150.0,
                    valores: "taxa",
                    lsc: tx("incidentes-lsc"),
                    lic: tx("incidentes-lic"),
                    nome_do_painel: false,
                    sinais: false,
                };
                utsim_posto_ui::carta::desenhar(ui, "carta_u", &ch, &estilo);
            }
        }
    }

    fn pessoas_ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        ui.label(tx("incidentes-dados_pessoais_ficam_so_na"));
        let aberta = self.restrita(s).is_some();
        if !aberta {
            ui.colored_label(VERMELHO, tx("incidentes-a_pasta_restrita_nao_existe"));
            ui.horizontal(|ui| {
                ui.label(tx("incidentes-pasta_fora_da_unidade_vazio"));
                utsim_ui_tema::arquivos::campo(
                    ui,
                    &mut self.caminho_restrita,
                    utsim_ui_tema::arquivos::Tipo::Pasta,
                    tx("incidentes-escolha_a_pasta_restrita"),
                    &[],
                    360.0,
                );
            });
            campo(
                ui,
                tx("incidentes-quem_pode_gravar_autores_separados"),
                &mut self.escritores,
            );
            if ui.button(tx("incidentes-criar_pasta_restrita")).clicked() {
                self.criar_restrita(s);
            }
            return;
        }
        let (pessoas, pode) = match self.restrita(s) {
            Some(r) => (r.pessoas(), r.pode_escrever()),
            None => return,
        };
        if ui.button(tx("incidentes-reler_a_pasta_restrita")).clicked() {
            self.reler_restrita();
        }
        egui::Grid::new("grade_pessoas")
            .striped(true)
            .show(ui, |ui| {
                for c in [
                    tx("incidentes-codigo"),
                    tx("incidentes-nome"),
                    tx("incidentes-setor"),
                    tx("incidentes-consentimento"),
                    "",
                ] {
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
                                    .small_button(tx("incidentes-confirmar_apaga_o_nome_para"))
                                    .clicked()
                                {
                                    let c = p.codigo.clone();
                                    self.confirmar = None;
                                    self.anonimizar(s, &c);
                                }
                                if ui.small_button(tx("incidentes-cancelar")).clicked() {
                                    self.confirmar = None;
                                }
                            });
                        } else if ui.small_button(tx("incidentes-anonimizar")).clicked() {
                            self.confirmar = Some(p.codigo.clone());
                        }
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
        if !pode {
            ui.colored_label(VERMELHO, tx("incidentes-voce_nao_esta_entre_as"));
            return;
        }
        ui.separator();
        ui.strong(tx("incidentes-cadastrar_pessoa"));
        campo(ui, tx("incidentes-nome"), &mut self.pessoa.nome);
        campo(ui, tx("incidentes-setor"), &mut self.pessoa.setor);
        ui.checkbox(
            &mut self.pessoa.consentimento,
            tx("incidentes-consentimento_registrado"),
        );
        if ui.button(tx("incidentes-cadastrar")).clicked() {
            self.cadastrar(s);
        }
    }
}
