// Composant de démonstration (façon Angular).

/** Composant d'affichage. */
@Component({ selector: "app-demo" })
export class Composant {
  /** Valeur courante. */
  readonly valeur = 1;
  private cache = 0;
  private service = inject(Service);

  /** Affiche la valeur. */
  afficher(): void {
    this.calculer();
  }

  private calculer(): number {
    return this.valeur * 2 + this.cache;
  }
}

function Component(_: unknown) {
  return (c: unknown) => c;
}

/** Service injecté dans le composant. */
export class Service {}

function inject<T>(c: new () => T): T {
  return new c();
}
