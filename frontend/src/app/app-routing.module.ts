import {NgModule} from '@angular/core';
import {ExtraOptions, RouterModule, Routes} from '@angular/router';
import {AppLayoutComponent} from './layout/app.layout.component';
import {DashboardComponent} from "../dashboard/dashboard.component";

const routerOptions: ExtraOptions = {
  anchorScrolling: 'enabled'
};

const routes: Routes = [
  {
    path: '', component: AppLayoutComponent,
    children: [
      {path: '', component: DashboardComponent},
      {path: '**', redirectTo: '/notfound'}
    ]
  }];

@NgModule({
  imports: [RouterModule.forRoot(routes, routerOptions)],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
