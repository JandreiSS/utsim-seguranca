//! UTSim Segurança do Trabalho (stage QS9 of `docs/plano_qualidade_e_seguranca.md`):
//! inventário de riscos por posto em cinco etapas, matriz de risco, exposição simulada e
//! relatório. Janela nativa (eframe/egui).
//!
//! O aviso legal ([`AVISO_LEGAL`]) fica fixo no rodapé de todas as telas.

pub mod app;
pub mod exposicao;
pub mod inventario;
pub mod matriz;
pub mod relatorio;

pub use app::{App, Tela};
pub use utsim_seguranca_core::AVISO_LEGAL;

pub const TITULO: &str = "UTSim Segurança do Trabalho";
