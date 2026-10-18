import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', component: AccueilComponent },
  { path: 'ancien', redirectTo: '', pathMatch: 'full' },
  { path: 'reglages', loadComponent: () => import('./pages/reglages/reglages.component').then((m) => m.ReglagesComponent) },
];
