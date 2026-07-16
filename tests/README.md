# Tests

Los tests de integración se nombran por módulo:

```text
mutex_test.rs
rwlock_test.rs
atomics_test.rs
```

En concurrencia, una prueba debe preferir resultados deterministas. Cuando se
quiera mostrar un bug por interleaving, el test debe acotar la ejecución y el
capítulo debe explicar por qué el comportamiento es difícil de observar siempre.
