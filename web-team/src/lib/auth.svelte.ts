// Store d'auth basé sur les runes Svelte 5. Reflète l'état du module
// `api.ts` (qui détient la vérité), exposé sous une forme observable
// pour les composants Svelte.

import {
  apiPost,
  clearTokens,
  getAccessToken,
  getRole,
  getSubjectId,
  restoreSession,
  setTokens,
  type Role,
  type Tokens
} from './api';

interface AuthSnapshot {
  loggedIn: boolean;
  role: Role | null;
  subjectId: string | null;
}

class AuthStore {
  // État réactif via runes.
  loggedIn = $state(false);
  role = $state<Role | null>(null);
  subjectId = $state<string | null>(null);
  loading = $state(true);

  /** Recharge l'état depuis api.ts (utile après un set/clear). */
  refresh(): void {
    const token = getAccessToken();
    this.loggedIn = token !== null;
    this.role = getRole();
    this.subjectId = getSubjectId();
  }

  /** Tente de restaurer une session via le refresh sessionStorage. */
  async hydrate(): Promise<void> {
    this.loading = true;
    try {
      await restoreSession();
    } finally {
      this.refresh();
      this.loading = false;
    }
  }

  async login(username: string, password: string, role: Role): Promise<void> {
    const tokens = await apiPost<Tokens>('/api/v1/login', {
      username,
      password,
      role
    });
    setTokens(tokens);
    this.refresh();
  }

  async enroll(
    code: string,
    password: string,
    fingerprint: string,
    displayName?: string
  ): Promise<void> {
    const tokens = await apiPost<Tokens>('/api/v1/enroll', {
      code,
      password,
      fingerprint,
      display_name: displayName
    });
    // /enroll ne renvoie pas explicitement `role` mais le sub est un user.
    setTokens({ ...tokens, role: 'user' });
    this.refresh();
  }

  logout(): void {
    clearTokens();
    this.refresh();
  }

  snapshot(): AuthSnapshot {
    return {
      loggedIn: this.loggedIn,
      role: this.role,
      subjectId: this.subjectId
    };
  }
}

export const auth = new AuthStore();
