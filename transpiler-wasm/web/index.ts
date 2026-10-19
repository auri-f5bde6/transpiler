import "./styles.css";

import {init_panic_hook, generate_output} from '../pkg';

init_panic_hook()

const code_input = document.getElementById("code")! as HTMLTextAreaElement;
const token_tab_content = document.getElementById("token-tab-content")!;
const pretty_print_tab_content = document.getElementById("pretty-print-tab-content")!;
const lmc_tab_content = document.getElementById("lmc-tab-content")!;
const transpile_button = document.getElementById("floating-arrow")!;
const slider = document.getElementById("slider")! as HTMLDivElement
const code_half = document.getElementById("code-half-container")! as HTMLDivElement;
const token_tab = document.getElementById("token-tab")!;
const pretty_print_tab = document.getElementById("pretty-print-tab")!;
const ast_tab = document.getElementById("ast-tab")!;
const lmc_tab = document.getElementById("lmc-tab")!;
const ast_tab_content = document.getElementById("ast-tab-content")!;
const folded_ast_tab = document.getElementById("folded-ast-tab")!;
const folded_ast_tab_content = document.getElementById("folded-ast-tab-content")!;

google.charts.load('current', {'packages': ['orgchart']});

function create_chart(element: Element, ast: string) {
    const data = new google.visualization.DataTable();
    data.addColumn('string', 'Name');
    data.addColumn('string', 'Parent');
    data.addColumn('string', 'Tooltip');
    data.addRows(JSON.parse(ast));
    const chart = new google.visualization.OrgChart(element);
    chart.draw(data, {allowHtml: true, nodeClass: "ast-node", selectedNodeClass: "ast-node-selected"})
}

function clicked() {
    const output = generate_output(code_input.value);
    token_tab_content.textContent = output.tokens;
    lmc_tab_content.textContent = output.lmc;
    pretty_print_tab_content.textContent = output.pretty_printed;

    create_chart(ast_tab_content, output.ast)
    create_chart(folded_ast_tab_content, output.folded_ast)
}

code_input.value = `\
number=5
result=1
while number>0
   result=result*number
   number=number-1
endwhile
// 5*4*3*2*1 = 120
print(result == 5*4*3*2*1) // (basic) constant folding by walking AST
`;
transpile_button.onclick = clicked;

// https://stackoverflow.com/a/55565128
slider.onmousedown = function dragMouseDown(e) {
    e.preventDefault();
    let initial = e.clientX;
    let initial_width = code_half.getBoundingClientRect().width;
    document.onmousemove = function onMouseMove(e) {
        code_half.style.width = `${Math.max(initial_width + e.clientX - initial, 0)}px`;
    }
    document.onmouseup = () => document.onmousemove = document.onmouseup = null;
}

function on_click_apply_selected(elm: HTMLElement) {

    const f = () => {
        reset_selected();
        elm.classList.add("selected");
        for_elm_in_class("tab-contents", elm1 => (elm1 as HTMLElement).style.display = "none");
        const content = document.getElementById(`${elm.id}-content`)!;
        content.style.display = "block";
    };
    elm.onclick = f;
    return f
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

on_click_apply_selected(token_tab);
on_click_apply_selected(pretty_print_tab);
on_click_apply_selected(ast_tab);
on_click_apply_selected(folded_ast_tab);
const setup_lmc = on_click_apply_selected(lmc_tab);
setup_lmc()