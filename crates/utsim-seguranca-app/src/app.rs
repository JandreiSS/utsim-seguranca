//! A janela: telas e o aviso legal fixo no rodapé.

use std::path::PathBuf;

use eframe::egui;

use utsim_posto_ui::{Marca, Saida, Sessao, TelaInicial, TelaModelo, TelaPostos};

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

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.sessao.atualizar_se_preciso(ui.ctx());
        egui::Panel::top("navegacao").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(crate::TITULO);
                ui.separator();
                let abas = [
                    (Tela::Inicial, "Início"),
                    (Tela::Modelo, "Modelo"),
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
                    ui.add_enabled_ui(aberta || t == Tela::Inicial, |ui| {
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
        egui::CentralPanel::default().show(ui, |ui| {
            if self.tela == Tela::Inicial {
                self.abertura(ui);
                return;
            }
            egui::ScrollArea::both().show(ui, |ui| match self.tela {
                Tela::Inicial => {}
                Tela::Postos => self.postos.ui(ui, &mut self.sessao),
                Tela::Modelo => self.modelo.ui(ui, &mut self.sessao),
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

impl App {
    /// A abertura: abrir ou criar a unidade, os recentes e onde fica salvo.
    fn abertura(&mut self, ui: &mut egui::Ui) {
        match self.inicial.ui(ui, &mut self.sessao, &MARCA) {
            Some(Saida::Aberta | Saida::Continuar) => {
                self.tela = if self.sessao.tem_modelo() {
                    Tela::Modelo
                } else {
                    Tela::Postos
                };
            }
            Some(Saida::Modelo(p)) => {
                let autor = self.sessao.config.autor.clone();
                match self.modelo.abrir(&p, &autor) {
                    Ok(()) => self.tela = Tela::Modelo,
                    Err(e) => self.sessao.falhar(e),
                }
            }
            Some(Saida::Acao(_)) | None => {}
        }
    }
}

/// O nome, a cor e o ícone do app na abertura (os mesmos do lançador).
pub const MARCA: Marca = Marca {
    nome: crate::TITULO,
    subtitulo: "Segurança do trabalho",
    cor: egui::Color32::from_rgb(0xF2, 0x8C, 0x1E),
    icone: Some(crate::ICONE),
    modelo: "Abra um arquivo .ron do simulador para ver e editar a segurança de cada nó.",
};

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        utsim_ui_tema::maximizar_no_inicio(ui.ctx());
        self.draw(ui);
    }
}
