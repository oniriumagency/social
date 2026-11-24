---
name: revisar
description: Revisa de forma independiente una rama o PR del proyecto Social contra su tarea, la definición de hecho y las invariantes. Úsala cuando el usuario pida revisar un PR, una rama o el trabajo de otra sesión. No corrige, solo informa.
---

# Revisión independiente

Actúas como revisor que no escribió el código. No corrijas nada: informa.

## Pasos

1. Identifica la tarea por el nombre de la rama o el título del PR. Lee su fila en `docs/12-plan-de-desarrollo.md` y los documentos que cita.
2. Lee el diff completo frente a `main`.
3. Comprueba, en este orden:
   1. **Invariantes** I-1 a I-15 de `ANTIGRAVITY.md`. Una violación es siempre un hallazgo crítico.
   2. **Criterio de aceptación** de la tarea: ¿hay una prueba automática que lo demuestra de verdad?
   3. **Definición de hecho** de `docs/13-flujo-agentico.md` §4.
   4. **Atajos silenciosos:** simulacros donde debía ir la pieza real, valores fijos, pruebas omitidas o debilitadas, excepciones tragadas.
   5. **Alcance:** ¿hay cambios fuera de la tarea o de la fase actual?
   6. **Coherencia** con los documentos: si el código y un documento difieren, es un hallazgo.
   7. **Privacidad de registros:** ¿se registra algo de la lista prohibida de `docs/10` §5?
4. Ejecuta `pnpm check` en la rama y anota el resultado real.

## Informe

Lista los hallazgos de mayor a menor severidad. Para cada uno: archivo y línea, qué ocurre, qué entrada o situación lo provoca, y qué regla o documento incumple. Termina con un veredicto: **aprobar**, **aprobar con cambios menores** o **devolver**.

Si no hay hallazgos, dilo, e indica qué comprobaste y qué no pudiste comprobar.
