//! UTSim Segurança do Trabalho (stage QS9 of `docs/plano_qualidade_e_seguranca.md`):
//! inventário de riscos por posto em cinco etapas, matriz de risco, exposição simulada e
//! relatório. Janela nativa (eframe/egui).
//!
//! O aviso legal ([`AVISO_LEGAL`]) fica fixo no rodapé de todas as telas.

pub mod app;
pub mod checklist;
pub mod epi;
pub mod ergonomia;
pub mod exposicao;
pub mod incidentes;
pub mod inventario;
pub mod matriz;
pub mod modelo;
pub mod relatorio;

pub use app::{App, Tela};
pub use utsim_seguranca_core::AVISO_LEGAL;

pub const TITULO: &str = "UTSim Segurança do Trabalho";

/// O ícone do app (o mesmo do lançador), na janela e na abertura.
pub const ICONE: &[u8] = include_bytes!("../../../assets/icone/familia/seguranca_256.png");
