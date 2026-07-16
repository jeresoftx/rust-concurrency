# Tests

Los tests de integracion se nombran por modulo:

```text
mutex_test.rs
rwlock_test.rs
atomics_test.rs
```

En concurrencia, una prueba debe preferir resultados deterministas. Cuando se
quiera mostrar un bug por interleaving, el test debe acotar la ejecucion y el
capitulo debe explicar por que el comportamiento es dificil de observar siempre.
