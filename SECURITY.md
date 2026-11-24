# Política de seguridad / Security policy

## Cómo reportar una vulnerabilidad

**No abras una incidencia pública.** Repórtala en privado con el formulario de GitHub:
[Report a vulnerability](https://github.com/oniriumagency/social/security/advisories/new).

Incluye, si puedes: qué componente afecta, cómo reproducirla, el impacto que ves y una
forma de contactarte. No incluyas datos reales de personas.

- **Respuesta inicial:** en 72 horas.
- **Divulgación coordinada:** publicamos el aviso cuando hay parche, o a los 90 días del
  reporte si no lo hay, lo que ocurra antes, salvo acuerdo con quien reporta.
- **Reconocimiento:** te citamos en el aviso si lo deseas.
- **Recompensas:** habrá programa a partir de la fase F2, con prioridad en el registro de
  humanidad y el ledger (`docs/10` §7). Hoy no hay recompensas.

Una llave pública para cifrar reportes fuera de GitHub se publicará en este archivo
antes de la alfa pública (F1).

## Alcance

Todo el código de este repositorio. Interesan en especial:

- Cualquier vía por la que un documento de identidad, una biometría, una coordenada,
  una llave privada o una IP asociada a una cuenta salga del dispositivo o se registre.
- Falsificación o aceptación indebida de eventos firmados (`crates/social-core`).
- Fallos en el uso de criptografía (`crates/social-crypto`).

Fuera de alcance: ataques de denegación de servicio por volumen, ingeniería social y
fallos de dependencias de terceros ya publicados (repórtalos a su proyecto).

## Versiones con soporte

El proyecto está en fase F0 (cimientos) y no tiene versiones publicadas. Se corrige
siempre sobre `main`.

---

**English.** Do not open public issues for vulnerabilities. Report them privately through
[GitHub's form](https://github.com/oniriumagency/social/security/advisories/new). We reply
within 72 hours and disclose after a fix or 90 days after the report, whichever comes
first. Do not include real personal data in reports.
