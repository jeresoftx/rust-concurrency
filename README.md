# Rust Concurrency

Repositorio del camino troncal de Jeresoft Academy para estudiar concurrencia en
Rust. Pertenece al Semestre 3 del plan de estudios junto con
`rust-database-internals` (RFC-0001 §10).

El objetivo no es solo mostrar APIs concurrentes. El objetivo es crear un
recurso educativo completo: cada primitiva debe explicar por qué existe, qué
problema resuelve, qué invariantes protege, qué alternativas tiene, cómo se
implementa o modela, cómo se prueba y cómo se mide.

## Qué Contiene

- Capítulos en Markdown compatibles con mdBook.
- Implementaciones y modelos Rust idiomáticos, un tema por módulo.
- Ejemplos progresivos: básico, intermedio, avanzado y caso real.
- Tests unitarios, tests de integracion y doctests.
- Benchmarks que confrontan el análisis teórico con mediciones.
- Diagramas Mermaid y recursos visuales.
- Ejercicios graduados con soluciones para niveles 1 a 3.

## Lugar En El Camino

Este curso vive en el Semestre 3. Recibe ideas de sistemas operativos,
estructuras de datos y Rust básico, y alimenta bases de datos internals,
sistemas distribuidos, travel tech, mensajería, video y performance.

La concurrencia es canónica aquí: `Mutex`, `RwLock`, atómicos, `Arc`, memory
ordering, deadlocks, channels, estructuras lock-free, hazard pointers y epoch GC
se explican en este repositorio antes de reutilizarse en cursos posteriores.

## Capítulos

| # | Capítulo | Módulo | Estado |
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

## Cómo Usarlo

Ejecutar tests:

```bash
cargo test
```

Formatear:

```bash
cargo fmt
```

Verificación completa:

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

- `AGENTS.md` es la guía de arranque para humanos e IA en este repositorio.
- `ROADMAP.md` registra el avance del curso sin convertirlo en una fecha límite.
- `docs/superpowers/plans/2026-07-16-rust-concurrency-course.md` contiene el
  checklist de implementación inicial.
- `LICENSE.md` resume la doble licencia: código bajo `MIT OR Apache-2.0`;
  contenido educativo bajo `CC BY-SA 4.0`.

## Filosofia

Este repositorio debe poder leerse como un libro de ingeniería. La claridad
gana sobre el ingenio, la calidad gana sobre la velocidad, y ningún capítulo se
considera publicable hasta cumplir la anatomía completa de RFC-0001 §14.
