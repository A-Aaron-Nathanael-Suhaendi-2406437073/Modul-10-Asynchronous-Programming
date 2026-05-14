use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
       <div class="bg-gradient-to-r from-cyan-500 to-blue-500 flex w-screen h-screen">
            <div class="container mx-auto flex flex-col justify-center items-center">
                <div class="text-white text-4xl font-bold mb-8 drop-shadow-md">{"Welcome to AeroChat 🚀"}</div>
                <form class="m-4 flex shadow-2xl rounded-lg">
                    <input {oninput} class="rounded-l-full p-4 border-t mr-0 border-b border-l text-gray-800 border-gray-200 bg-white w-64 focus:outline-none focus:ring-2 focus:ring-cyan-300" placeholder="Enter your nickname..." />
                    <Link<Route> to={Route::Chat}> <button {onclick} disabled={username.len()<1} class="px-8 rounded-r-full bg-gray-900 hover:bg-gray-800 transition-colors duration-200 text-white font-bold p-4 uppercase border-gray-900 border-t border-b border-r disabled:opacity-50" >{"Join Room"}</button></Link<Route>>
                </form>
            </div>
        </div>
    }
}
