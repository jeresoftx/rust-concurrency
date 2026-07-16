# ROADMAP

Estado de avance de `rust-concurrency`, repositorio del camino troncal de
Jeresoft Academy para concurrencia en Rust.

No hay fechas limite: este es un proyecto de legado (RFC-0001 §1). Este archivo
orienta el avance, pero no convierte el curso en una carrera por terminar.

## Estado Actual

El repositorio acaba de nacer. La primera meta es crear la fundacion del curso:
identidad, crate Rust, estructura de documentacion, CI y un plan de trabajo
ejecutable.

El checklist detallado vive en
[`docs/superpowers/plans/2026-07-16-rust-concurrency-course.md`](docs/superpowers/plans/2026-07-16-rust-concurrency-course.md).

## Capitulos Planeados

| # | Capitulo | Estado |
|---|----------|--------|
| 01 | Mutex | planned |
| 02 | RwLock | planned |
| 03 | Atomics | planned |
| 04 | Arc | planned |
| 05 | Memory Ordering | planned |
| 06 | Deadlocks | planned |
| 07 | Channels | planned |
| 08 | Lock-Free Structures | planned |
| 09 | Hazard Pointers | planned |
| 10 | Epoch GC | planned |

## Alineacion RFC-0001

- Este repositorio sigue la plantilla de repositorio de RFC-0001 §15.
- Cada capitulo debe cumplir la anatomia de RFC-0001 §14.
- Cada ejercicio debe seguir los niveles de RFC-0001 §17.
- El uso de IA se rige por RFC-0001 §20: la IA acelera, el criterio humano
  decide.

## Fuera De Alcance Por Ahora

- Async programming con Tokio: vive en `rust-async`, salvo comparaciones
  necesarias.
- Sistemas distribuidos: viven en `rust-distributed-systems`, salvo notas de
  camino.
- Operating systems internals profundos: viven en `rust-operating-systems`.
- Implementaciones `unsafe` avanzadas sin justificacion escrita y revision
  humana explicita.
