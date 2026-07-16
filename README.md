# Rust Concurrency

Repositorio del camino troncal de Jeresoft Academy para estudiar concurrencia en
Rust. Pertenece al Semestre 3 del plan de estudios junto con
`rust-database-internals` (RFC-0001 §10).

El objetivo no es solo mostrar APIs concurrentes. El objetivo es crear un
recurso educativo completo: cada primitiva debe explicar por que existe, que
problema resuelve, que invariantes protege, que alternativas tiene, como se
implementa o modela, como se prueba y como se mide.

## Que Contiene

- Capitulos en Markdown compatibles con mdBook.
- Implementaciones y modelos Rust idiomaticos, un tema por modulo.
- Ejemplos progresivos: basico, intermedio, avanzado y caso real.
- Tests unitarios, tests de integracion y doctests.
- Benchmarks que confrontan el analisis teorico con mediciones.
- Diagramas Mermaid y recursos visuales.
- Ejercicios graduados con soluciones para niveles 1 a 3.

## Lugar En El Camino

Este curso vive en el Semestre 3. Recibe ideas de sistemas operativos,
estructuras de datos y Rust basico, y alimenta bases de datos internals,
sistemas distribuidos, travel tech, mensajeria, video y performance.

La concurrencia es canonica aqui: `Mutex`, `RwLock`, atomicos, `Arc`, memory
ordering, deadlocks, channels, estructuras lock-free, hazard pointers y epoch GC
se explican en este repositorio antes de reutilizarse en cursos posteriores.

## Capitulos

| # | Capitulo | Modulo | Estado |
|---|----------|--------|--------|
| 01 | Mutex | `src/mutex.rs` | planned |
| 02 | RwLock | `src/rwlock.rs` | planned |
| 03 | Atomics | `src/atomics.rs` | planned |
| 04 | Arc | `src/arc.rs` | planned |
| 05 | Memory Ordering | `src/memory_ordering.rs` | planned |
| 06 | Deadlocks | `src/deadlocks.rs` | planned |
| 07 | Channels | `src/channels.rs` | planned |
| 08 | Lock-Free Structures | `src/lock_free.rs` | planned |
| 09 | Hazard Pointers | `src/hazard_pointers.rs` | planned |
| 10 | Epoch GC | `src/epoch_gc.rs` | planned |

Estados posibles: `planned`, `draft`, `implemented`, `tested`,
`benchmarked`, `reviewed`, `published`.

## Estructura Esperada

```text
AGENTS.md
ROADMAP.md
LICENSE.md
LICENSE-MIT
LICENSE-APACHE
LICENSE-CC-BY-SA-4.0.md
docs/
  SUMMARY.md
src/
  lib.rs
  mutex.rs
examples/
  soluciones/
tests/
benches/
diagrams/
assets/
```

## Como Usarlo

Ejecutar tests:

```bash
cargo test
```

Formatear:

```bash
cargo fmt
```

Verificacion completa:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --doc
```

Ejecutar benchmarks:

```bash
cargo bench
```

## Gobernanza

- `AGENTS.md` es la guia de arranque para humanos e IA en este repositorio.
- `ROADMAP.md` registra el avance del curso sin convertirlo en una fecha limite.
- `docs/superpowers/plans/2026-07-16-rust-concurrency-course.md` contiene el
  checklist de implementacion inicial.
- `LICENSE.md` resume la doble licencia: codigo bajo `MIT OR Apache-2.0`;
  contenido educativo bajo `CC BY-SA 4.0`.

## Filosofia

Este repositorio debe poder leerse como un libro de ingenieria. La claridad
gana sobre el ingenio, la calidad gana sobre la velocidad, y ningun capitulo se
considera publicable hasta cumplir la anatomia completa de RFC-0001 §14.
