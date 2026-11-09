//! A janela: telas e o aviso legal fixo no rodapé.

use std::path::PathBuf;

use eframe::egui;

use utsim_posto_ui::{Sessao, TelaFerramentas, TelaInicial, TelaModelo, TelaPostos};

use crate::checklist::TelaChecklist;
use crate::epi::TelaEpi;
use crate::ergonomia::TelaErgonomia;
use crate::exposicao::TelaExposicao;
use crate::incidentes::TelaIncidentes;
use crate::inventario::TelaInventario;
use crate::matriz::TelaMatriz;
use crate::relatorio::TelaRelatorio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tela {
    Inicial,
    Postos,
    Modelo,
    Ferramentas,
    Inventario,
    Matriz,
    Exposicao,
    Ergonomia,
    Checklist,
    Epi,
    Incidentes,
    Relatorio,
}

pub struct App {
    pub sessao: Sessao,
    pub tela: Tela,
    pub inicial: TelaInicial,
    pub postos: TelaPostos,
    pub modelo: TelaModelo,
    pub ferramentas: TelaFerramentas,
    pub inventario: TelaInventario,
    pub matriz: TelaMatriz,
    pub exposicao: TelaExposicao,
    pub ergonomia: TelaErgonomia,
    pub checklist: TelaChecklist,
    pub epi: TelaEpi,
    pub incidentes: TelaIncidentes,
    pub relatorio: TelaRelatorio,
}

impl App {
    pub fn new(config: Option<PathBuf>) -> App {
        App {
            sessao: Sessao::nova(config),
            tela: Tela::Inicial,
            inicial: TelaInicial::default(),
            postos: TelaPostos::default(),
            modelo: crate::modelo::tela(),
            ferramentas: TelaFerramentas::nova(
                utsim_estudo::App::Seguranca,
                crate::identidade(),
                None,
            ),
            inventario: TelaInventario::default(),
            matriz: TelaMatriz::default(),
            exposicao: TelaExposicao::default(),
            ergonomia: TelaErgonomia::default(),
            checklist: TelaChecklist::default(),
            epi: TelaEpi::default(),
            incidentes: TelaIncidentes::default(),
            relatorio: TelaRelatorio::default(),
        }
    }

    /// Atende o pedido da linha de comando (`--utsim-abrir`, seção 2.2 do plano de
    /// integração): o modelo no nó, ou os dados numa ferramenta.
    pub fn atender(&mut self, a: &utsim_posto_ui::Abrir) {
        match utsim_posto_ui::abrir::atender(
            a,
            &mut self.sessao,
            &mut self.modelo,
            &mut self.ferramentas,
        ) {
            Some(utsim_posto_ui::Destino::Modelo) => self.tela = Tela::Modelo,
            Some(utsim_posto_ui::Destino::Ferramentas) => self.tela = Tela::Ferramentas,
            None => {}
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.sessao.atualizar_se_preciso(ui.ctx());
        egui::Panel::top("navegacao").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(crate::TITULO);
                ui.separator();
                let abas = [
                    (Tela::Inicial, "Unidade"),
                    (Tela::Modelo, "Modelo"),
                    (Tela::Ferramentas, "Ferramentas"),
                    (Tela::Postos, "Postos"),
                    (Tela::Inventario, "Inventário de riscos"),
                    (Tela::Matriz, "Matriz"),
                    (Tela::Exposicao, "Exposição"),
                    (Tela::Ergonomia, "Ergonomia"),
                    (Tela::Checklist, "Checklist"),
                    (Tela::Epi, "EPI"),
                    (Tela::Incidentes, "Incidentes"),
                    (Tela::Relatorio, "Relatório"),
                ];
                let aberta = self.sessao.unidade.is_some();
                for (t, nome) in abas {
                    let modelo_aberto = t == Tela::Modelo && self.modelo.aberto();
                    ui.add_enabled_ui(aberta || modelo_aberto || t == Tela::Inicial, |ui| {
                        if ui.selectable_label(self.tela == t, nome).clicked() {
                            self.tela = t;
                        }
                    });
                }
            });
        });
        // O aviso legal aparece em TODAS as telas.
        egui::Panel::bottom("aviso_legal").show(ui, |ui| {
            ui.colored_label(egui::Color32::from_rgb(230, 160, 40), crate::AVISO_LEGAL);
        });
        egui::Panel::bottom("estado").show(ui, |ui| self.sessao.barra(ui));
        if let Some((modelo, objeto)) = self.modelo.pedido_ferramentas.take() {
            self.ferramentas.abrir_no(&modelo, &objeto);
            self.tela = Tela::Ferramentas;
        }
        if self.tela != Tela::Ferramentas {
            self.ferramentas.gravar_pendente(&mut self.sessao);
        }
        egui::CentralPanel::default().show(ui, |ui| {
            if self.tela == Tela::Ferramentas {
                // A tela tem as próprias barras de rolagem (lista e ferramenta).
                self.ferramentas.ui(ui, &mut self.sessao);
                return;
            }
            egui::ScrollArea::both().show(ui, |ui| match self.tela {
                Tela::Inicial => {
                    if self.inicial.ui(ui, &mut self.sessao) {
                        self.tela = if self.sessao.tem_modelo() {
                            Tela::Modelo
                        } else {
                            Tela::Postos
                        };
                    }
                }
                Tela::Postos => self.postos.ui(ui, &mut self.sessao),
                Tela::Modelo => self.modelo.ui(ui, &mut self.sessao),
                Tela::Ferramentas => {}
                Tela::Inventario => self.inventario.ui(ui, &mut self.sessao),
                Tela::Matriz => self.matriz.ui(ui, &mut self.sessao),
                Tela::Exposicao => self.exposicao.ui(ui, &mut self.sessao),
                Tela::Ergonomia => self.ergonomia.ui(ui, &mut self.sessao),
                Tela::Checklist => self.checklist.ui(ui, &mut self.sessao),
                Tela::Epi => self.epi.ui(ui, &mut self.sessao),
                Tela::Incidentes => self.incidentes.ui(ui, &mut self.sessao),
                Tela::Relatorio => self.relatorio.ui(ui, &mut self.sessao),
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.ferramentas.gravar_pendente(&mut self.sessao);
    }
}
