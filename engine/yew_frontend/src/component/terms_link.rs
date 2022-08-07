use crate::component::route_link::RouteLink;
use crate::translation::{t, Translation};
use crate::Route;
use yew::{function_component, html};

#[function_component(TermsLink)]
pub fn terms_link() -> Html {
    html! {
        <RouteLink<Route> route={Route::Terms}>{t().terms_hint()}</RouteLink<Route>>
    }
}