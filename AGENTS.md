# AGENTS.md

Este repositorio es parte de la coleccion camino troncal / Semestre 3 de
Jeresoft Academy y se rige por la RFC-0001 (manual fundacional).

## Objetivo

Crear el mejor recurso educativo posible sobre concurrencia en Rust.

Todo cambio debe mejorar simultaneamente:

- calidad tecnica
- claridad
- documentacion
- mantenibilidad

## Antes de escribir codigo

Siempre, en este orden (RFC-0001 §13):

1. Explicar el concepto.
2. Explicar el problema.
3. Comparar alternativas.
4. Justificar la implementacion.

## Codigo

Conforme a RFC-0001 §13:

- Rust idiomatico.
- Clippy limpio y rustfmt sin diffs.
- Sin `unsafe` salvo justificacion documentada con comentario `// SAFETY:`.
- Comentarios donde aporten valor.
- Tests concurrentes deben ser deterministas cuando sea posible; si una prueba
  depende de interleavings, el capitulo debe explicar la limitacion.

## Documentacion

Todo capitulo sigue las doce secciones de RFC-0001 §14 y la plantilla de §16.
Toda nueva funcionalidad incluye:

- README actualizado.
- Diagramas Mermaid (RFC-0001 §12).
- Ejemplos ejecutables.
- Tests.
- Benchmarks cuando apliquen; si no aplican, se declara.

## Nunca

- Agregar dependencias innecesarias.
- Optimizar prematuramente.
- Duplicar codigo.
- Omitir documentacion.
- Publicar capitulos parciales.
- Presentar concurrencia como magia: toda primitiva debe declarar invariantes,
  garantias y modos de falla.

## Filosofia

Este repositorio debe poder utilizarse como un libro de ingenieria. Nunca
sacrificar claridad por ingenio. Explicar el porque, no solo el como.
