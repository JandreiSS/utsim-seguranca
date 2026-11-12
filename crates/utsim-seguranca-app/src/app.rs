//! A janela: telas e o aviso legal fixo no rodapé.

use crate::texto::tx;
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
        let sessao = Sessao::nova(config);
        // A língua guardada; sem ela, a de agora (a do sistema, posta pelo `main`).
        if let Some(i) = sessao
            .config
            .idioma
            .as_deref()
            .and_then(utsim_idiomas::Idioma::de_codigo)
        {
            utsim_idiomas::definir(i);
        }
        App {
            sessao,
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
        let motivo = tx("app-abra_uma_unidade_no_inicio");
        let it = |t, nome| Item::novo(t, nome).bloqueado_se(sem, motivo);
        let grupos = [
            Grupo {
                titulo: None,
                itens: vec![Item::novo(Tela::Inicial, tx("app-inicio"))],
            },
            Grupo {
                titulo: Some(tx("app-modelo_e_postos")),
                itens: vec![
                    Item::novo(Tela::Modelo, tx("app-modelo"))
                        .bloqueado_se(sem && !self.modelo.aberto(), motivo),
                    it(Tela::Postos, tx("app-postos")),
                ],
            },
            Grupo {
                titulo: Some(tx("app-ferramentas_e_estudos")),
                itens: vec![it(Tela::Ferramentas, tx("app-ferramentas"))],
            },
            Grupo {
                titulo: Some(tx("app-analises")),
                itens: vec![
                    it(Tela::Exposicao, tx("app-exposicao")),
                    it(Tela::Ergonomia, tx("app-ergonomia")),
                    it(Tela::Matriz, tx("app-matriz_de_risco")),
                ],
            },
            Grupo {
                titulo: Some(tx("app-acoes_e_registros")),
                itens: vec![
                    it(Tela::Inventario, tx("app-inventario_de_riscos")),
                    it(Tela::Checklist, tx("app-checklist")),
                    it(Tela::Epi, tx("app-epi")),
                    it(Tela::Incidentes, tx("app-incidentes")),
                ],
            },
            Grupo {
                titulo: Some(tx("app-relatorio")),
                itens: vec![it(Tela::Relatorio, tx("app-relatorio"))],
            },
        ];
        let linhas = utsim_posto_ui::linhas_da_barra(&self.sessao, &self.modelo);
        Barra {
            nome: crate::TITULO,
            cor: COR_APP,
            icone: Some(crate::ICONE),
            linhas: &linhas,
            grupos: &grupos,
        }
        .mostrar(ui, &mut self.tela);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let mut i = utsim_idiomas::atual();
            if utsim_idiomas::seletor(ui, "idioma", &mut i) {
                utsim_idiomas::definir(i);
                self.sessao.config.idioma = Some(i.codigo().to_string());
                self.sessao.gravar_config();
            }
        });
    }

    /// A abertura: abrir ou criar a unidade, os recentes e onde fica salvo.
    fn abertura(&mut self, ui: &mut egui::Ui) {
        match self.inicial.ui(ui, &mut self.sessao, &marca()) {
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

/// A cor do app (a mesma do lançador).
pub const COR_APP: egui::Color32 = egui::Color32::from_rgb(0xF2, 0x8C, 0x1E);

/// O nome, a cor e o ícone do app na abertura (os mesmos do lançador), na língua de agora.
pub fn marca() -> Marca {
    Marca {
        nome: crate::TITULO,
        subtitulo: tx("app-seguranca_do_trabalho"),
        cor: COR_APP,
        icone: Some(crate::ICONE),
        modelo: tx("app-abra_um_arquivo_ron_do"),
    }
}

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
