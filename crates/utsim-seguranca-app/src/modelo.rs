//! Aba "Modelo" do UTSim Segurança do Trabalho: edita `ppe` e `safety` de cada nó do
//! `.ron` do UTSim e mostra ao lado a exposição simulada das pessoas que passam no nó.

use std::collections::BTreeMap;
use std::sync::Arc;

use utsim_edit::abas::Aba;
use utsim_exposicao::{Options, analyze_all, contains};
use utsim_model::Model;
use utsim_posto_ui::{DESTAQUE, Resumo, TelaModelo};

/// Soma por pessoa: tempo em zona, entradas sem EPI, quase-conflitos.
#[derive(Default)]
struct Soma {
    tempo: f64,
    sem_epi: f64,
    quase: f64,
}

/// Exposição simulada por nó. As pessoas de um nó são as que passam por zonas que
/// contêm a posição do nó (e, para um nó que é a própria pessoa, as zonas por onde ela
/// passou). Os quase-conflitos contam só os que acontecem dentro dessas zonas, ou de que o
/// nó participa. Médias por replicação.
pub fn resumo(modelo: &Model) -> Result<Resumo, String> {
    let reps = analyze_all(modelo, Options::default()).map_err(|e| e.to_string())?;
    let n = reps.len().max(1) as f64;
    let mut saida: Resumo = BTreeMap::new();
    let total_quase: f64 = reps.iter().map(|e| e.near_miss_count() as f64).sum::<f64>() / n;
    let total_sem_epi: f64 = reps
        .iter()
        .map(|e| f64::from(e.entries_without_ppe()))
        .sum::<f64>()
        / n;
    saida.insert(
        String::new(),
        vec![
            (
                "Quase-conflitos (média)".into(),
                format!("{total_quase:.2}"),
            ),
            (
                "Entradas sem EPI (média)".into(),
                format!("{total_sem_epi:.2}"),
            ),
            ("Origem".into(), "Simulado".into()),
        ],
    );
    for o in &modelo.objects {
        let zonas: Vec<&str> = modelo
            .zones
            .iter()
            .filter(|z| contains(&z.polygon, [o.position[0], o.position[1]]))
            .map(|z| z.name.as_str())
            .collect();
        let mut por_pessoa: BTreeMap<String, Soma> = BTreeMap::new();
        for e in &reps {
            for z in &e.zones {
                if zonas.contains(&z.zone.as_str()) || z.person == o.name {
                    let s = por_pessoa.entry(z.person.clone()).or_default();
                    s.tempo += z.time / n;
                    s.sem_epi += f64::from(z.entries_without_ppe) / n;
                }
            }
            // Only the near conflicts that happen inside the zones of the node, or that the
            // node itself (a person or a vehicle) takes part in.
            for q in &e.near_misses {
                let aqui = q.person == o.name
                    || q.vehicle == o.name
                    || modelo
                        .zones
                        .iter()
                        .any(|z| zonas.contains(&z.name.as_str()) && contains(&z.polygon, q.at));
                if aqui {
                    por_pessoa.entry(q.person.clone()).or_default().quase += 1.0 / n;
                }
            }
        }
        if por_pessoa.is_empty() {
            continue;
        }
        let quase: f64 = por_pessoa.values().map(|s| s.quase).sum();
        let sem_epi: f64 = por_pessoa.values().map(|s| s.sem_epi).sum();
        let mut linhas = vec![(
            DESTAQUE.to_string(),
            format!("{quase:.2} quase-conflitos, {sem_epi:.2} sem EPI"),
        )];
        linhas.extend(por_pessoa.into_iter().map(|(p, s)| {
            (
                p,
                format!(
                    "{:.1} s em zona, {:.2} entradas sem EPI, {:.2} quase-conflitos",
                    s.tempo, s.sem_epi, s.quase
                ),
            )
        }));
        saida.insert(o.name.clone(), linhas);
    }
    Ok(saida)
}

/// A tela do app: aba Segurança editável, Manutenção só leitura.
pub fn tela() -> TelaModelo {
    TelaModelo::nova("seguranca", Aba::Seguranca, Arc::new(resumo))
}
