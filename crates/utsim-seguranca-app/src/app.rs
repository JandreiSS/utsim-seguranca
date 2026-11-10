//! A janela: telas e o aviso legal fixo no rodapé.

use std::path::PathBuf;

use eframe::egui;

use utsim_posto_ui::{Marca, Saida, Sessao, TelaFerramentas, TelaInicial, TelaModelo, TelaPostos};

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
    /// O pedido de outro app (`--utsim-pedido`), respondido ao fechar.
    pub pedido: Option<utsim_posto_ui::abrir::Atendido>,
}

impl App {
    pub fn new(config: Option<PathBuf>) -> App {
        App {
            sessao: Sessao::nova(config),
            tela: Tela::Inicial,
            pedido: None,
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
        let destino = utsim_posto_ui::abrir::atender(
            a,
            &mut self.sessao,
            &mut self.modelo,
            &mut self.ferramentas,
        );
        self.pedido = utsim_posto_ui::abrir::Atendido::de(a, &self.ferramentas);
        match destino {
            Some(utsim_posto_ui::Destino::Modelo) => self.tela = Tela::Modelo,
            Some(utsim_posto_ui::Destino::Ferramentas) => self.tela = Tela::Ferramentas,
            None => {}
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.sessao.atualizar_se_preciso(ui.ctx());
        egui::Panel::left("navegacao")
            .default_size(utsim_ui_tema::navegacao::LARGURA)
            .resizable(false)
            .show(ui, |ui| self.barra_lateral(ui));
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
            if self.tela == Tela::Modelo {
                // Três zonas, cada uma com a própria rolagem.
                self.modelo.ui(ui, &mut self.sessao);
                return;
            }
            if self.tela == Tela::Inicial {
                self.abertura(ui);
                return;
            }
            egui::ScrollArea::both().show(ui, |ui| match self.tela {
                Tela::Inicial => {}
                Tela::Postos => self.postos.ui(ui, &mut self.sessao),
                Tela::Modelo | Tela::Ferramentas => {}
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
    /// A barra lateral: o app, a unidade aberta e as telas em grupos. Sem unidade, só o
    /// Início fica ativo (e o Modelo, quando há um modelo aberto pelo 3D).
    fn barra_lateral(&mut self, ui: &mut egui::Ui) {
        use utsim_ui_tema::navegacao::{Barra, Grupo, Item};
        let sem = self.sessao.unidade.is_none();
        let motivo = "Abra uma unidade no Início.";
        let it = |t, nome| Item::novo(t, nome).bloqueado_se(sem, motivo);
        let grupos = [
            Grupo {
                titulo: None,
                itens: vec![Item::novo(Tela::Inicial, "Início")],
            },
            Grupo {
                titulo: Some("Modelo e postos"),
                itens: vec![
                    Item::novo(Tela::Modelo, "Modelo")
                        .bloqueado_se(sem && !self.modelo.aberto(), motivo),
                    it(Tela::Postos, "Postos"),
                ],
            },
            Grupo {
                titulo: Some("Ferramentas e estudos"),
                itens: vec![it(Tela::Ferramentas, "Ferramentas")],
            },
            Grupo {
                titulo: Some("Análises"),
                itens: vec![
                    it(Tela::Exposicao, "Exposição"),
                    it(Tela::Ergonomia, "Ergonomia"),
                    it(Tela::Matriz, "Matriz de risco"),
                ],
            },
            Grupo {
                titulo: Some("Ações e registros"),
                itens: vec![
                    it(Tela::Inventario, "Inventário de riscos"),
                    it(Tela::Checklist, "Checklist"),
                    it(Tela::Epi, "EPI"),
                    it(Tela::Incidentes, "Incidentes"),
                ],
            },
            Grupo {
                titulo: Some("Relatório"),
                itens: vec![it(Tela::Relatorio, "Relatório")],
            },
        ];
        let linhas = utsim_posto_ui::linhas_da_barra(&self.sessao, &self.modelo);
        Barra {
            nome: crate::TITULO,
            cor: MARCA.cor,
            icone: MARCA.icone,
            linhas: &linhas,
            grupos: &grupos,
        }
        .mostrar(ui, &mut self.tela);
    }

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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.ferramentas.gravar_pendente(&mut self.sessao);
        if let Some(p) = &self.pedido
            && let Some(Err(e)) = p.responder(
                &self.ferramentas,
                &self.sessao,
                "seguranca",
                env!("CARGO_PKG_VERSION"),
            )
        {
            eprintln!("resposta do pedido: {e}");
        }
    }
}
