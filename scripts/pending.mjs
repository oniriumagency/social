#!/usr/bin/env node
// Marca una orden de docs/11 §3 que todavía no tiene implementación.
// F0-T01 exige que todas las órdenes existan aunque estén vacías; cada una la
// completa la tarea del plan indicada. Sale con éxito para no bloquear `pnpm check`.

const [order, task] = process.argv.slice(2);
console.log(`"pnpm ${order}" todavía no hace nada: la implementa la tarea ${task} (docs/12).`);
