import {NgModule} from "@angular/core";
import {AppLayoutComponent} from "./app.layout.component";
import {AppBreadcrumbComponent} from "./app.breadcrumb.component";
import {AppSidebarComponent} from "./app.sidebar.component";
import {AppTopbarComponent} from "./app.topbar.component";
import {AppRightMenuComponent} from "./app.rightmenu.component";
import {AppMenuComponent} from "./app.menu.component";
import {AppMenuitemComponent} from "./app.menuitem.component";
import {AppFooterComponent} from "./app.footer.component";
import {AppSearchComponent} from "./app.search.component";
import {FormsModule} from "@angular/forms";
import {InputTextModule} from "primeng/inputtext";
import {SidebarModule} from "primeng/sidebar";
import {BadgeModule} from "primeng/badge";
import {RadioButtonModule} from "primeng/radiobutton";
import {InputSwitchModule} from "primeng/inputswitch";
import {ButtonModule} from "primeng/button";
import {TooltipModule} from "primeng/tooltip";
import {RippleModule} from "primeng/ripple";
import {MenuModule} from "primeng/menu";
import {RouterModule} from "@angular/router";
import {DropdownModule} from "primeng/dropdown";
import {DividerModule} from "primeng/divider";
import {AppConfigModule} from "./config/app.config.module";
import {DialogModule} from "primeng/dialog";
import {StyleClassModule} from "primeng/styleclass";
import {CommonModule} from "@angular/common";
import {BrowserModule} from "@angular/platform-browser";
import {BrowserAnimationsModule} from "@angular/platform-browser/animations";


@NgModule({
  declarations: [
    AppLayoutComponent,
    AppBreadcrumbComponent,
    AppSidebarComponent,
    AppTopbarComponent,
    AppRightMenuComponent,
    AppMenuComponent,
    AppMenuitemComponent,
    AppSearchComponent,
    AppFooterComponent
  ],
  imports: [
    FormsModule,
    BrowserModule,
    BrowserAnimationsModule,
    CommonModule,
    InputTextModule,
    SidebarModule,
    BadgeModule,
    RadioButtonModule,
    InputSwitchModule,
    ButtonModule,
    TooltipModule,
    RippleModule,
    MenuModule,
    RouterModule,
    DropdownModule,
    DividerModule,
    AppConfigModule,
    DialogModule,
    StyleClassModule
  ]
})
export class AppLayoutModule {
}
