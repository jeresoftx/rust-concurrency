# AGENTS.md

Este repositorio es parte de la colección camino troncal / Semestre 3 de
Jeresoft Academy y se rige por la RFC-0001 (manual fundacional).

## Objetivo

Crear el mejor recurso educativo posible sobre concurrencia en Rust.

Todo cambio debe mejorar simultáneamente:

- calidad técnica
- claridad
- documentación
- mantenibilidad

## Antes de escribir código

Siempre, en este orden (RFC-0001 §13):

1. Explicar el concepto.
2. Explicar el problema.
3. Comparar alternativas.
4. Justificar la implementación.

## Código

Conforme a RFC-0001 §13:

- Rust idiomático.
- Clippy limpio y rustfmt sin diffs.
- Sin `unsafe` salvo justificación documentada con comentario `// SAFETY:`.
- Comentarios donde aporten valor.
- Tests concurrentes deben ser deterministas cuando sea posible; si una prueba
  depende de interleavings, el capítulo debe explicar la limitación.

## Documentación

Todo capítulo sigue las doce secciones de RFC-0001 §14 y la plantilla de §16.
Toda nueva funcionalidad incluye:

- README actualizado.
- Diagramas Mermaid (RFC-0001 §12).
- Ejemplos ejecutables.
- Tests.
- Benchmarks cuando apliquen; si no aplican, se declara.

## Nunca

- Agregar dependencias innecesarias.
- Optimizar prematuramente.
- Duplicar código.
- Omitir documentación.
- Publicar capítulos parciales.
- Presentar concurrencia como magia: toda primitiva debe declarar invariantes,
  garantías y modos de falla.

## Filosofía

Este repositorio debe poder utilizarse como un libro de ingeniería. Nunca
sacrificar claridad por ingenio. Explicar el porqué, no solo el cómo.
