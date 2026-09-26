import "./styles.css";

import {pretty_print, transpile} from '../pkg';

const code_input = document.getElementById("code")! as HTMLTextAreaElement;
const pretty_print_tab_content = document.getElementById("pretty-print-tab-content")!;
const lmc_tab_content = document.getElementById("lmc-tab-content")!;
const transpile_button = document.getElementById("floating-arrow")!;
const slider = document.getElementById("slider")! as HTMLDivElement
const code_half = document.getElementById("code-half-container")! as HTMLDivElement;
const pretty_print_tab = document.getElementById("pretty-print-tab")!;
const ast_tab = document.getElementById("ast-tab")!;
const lmc_tab = document.getElementById("lmc-tab")!;


function clicked() {
    lmc_tab_content.textContent = transpile(code_input.value);
    pretty_print_tab_content.textContent = pretty_print(code_input.value);
}

code_input.value = `\
number=5
result=1
while number>0
   result=result*number
   number=number-1
endwhile
// 5*4*3*2*1 = 120
print(result)
`;
transpile_button.onclick = clicked;

// https://stackoverflow.com/a/55565128
slider.onmousedown = function dragMouseDown(e) {
    e.preventDefault();
    let initial = e.clientX;
    let initial_width = code_half.getBoundingClientRect().width;
    document.onmousemove = function onMouseMove(e) {
        code_half.style.width = `${initial_width + e.clientX - initial}px`;
    }
    document.onmouseup = () => document.onmousemove = document.onmouseup = null;
}

function on_click_apply_selected(elm: HTMLElement) {
    elm.onclick = () => {
        reset_selected();
        elm.classList.add("selected");
        for_elm_in_class("tab-contents", elm1 => (elm1 as HTMLElement).style.display = "none");
        const content = document.getElementById(`${elm.id}-content`)!;
        content.style.display = "block";
    }
}

function for_elm_in_class(class_name: string, func: (elm: Element) => void) {
    const selected = document.getElementsByClassName(class_name);
    for (let i = 0; i < selected.length; i++) {
        const element = selected.item(i)!;
        func(element);
    }
}

function reset_selected(): void {
    for_elm_in_class("selected", elm => elm.classList.remove("selected"));
}

on_click_apply_selected(pretty_print_tab);
on_click_apply_selected(ast_tab);
on_click_apply_selected(lmc_tab);