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
/// A cor do app (capa dos relatórios e launcher).
pub const COR: &str = "#F28C1E";
/// O ícone do app (o mesmo do lançador), na janela, na abertura e na capa dos relatórios.
pub const ICONE: &[u8] = include_bytes!("../../../assets/icone/familia/seguranca_256.png");

/// A identidade do app nos relatórios dos estudos.
pub fn identidade() -> utsim_estudo::relatorio::Identidade {
    utsim_estudo::relatorio::Identidade {
        app: TITULO.into(),
        cor: COR.into(),
        icone_png: ICONE.to_vec(),
    }
}
