use crate::components::modal::ModalDialog;
use yew::prelude::*;

const ALG_SHEET_LINKS: [(&str, &str); 21] = [
    ("Skewb Resources", "https://docs.google.com/spreadsheets/d/1j-SGlgZk20D3d2TyeiXhMWzeyTBuuix3Owgj9Q8rNbo/edit#gid=1373252438"),
    ("Skewb Algs", "https://docs.google.com/spreadsheets/d/175ZOo4a9MwglJCnoJ3bWkPOhLhG__kPX4uVSkJMHwio/edit#gid=0"),
    ("The Pyraminx Sheet", "https://docs.google.com/spreadsheets/d/12FH8MswpdBJxLJ_Eu_gopoxTCySPk9H_o7_yjN8_ECw/edit#gid=566431744"),
    ("James' Megaminx Algs", "https://docs.google.com/spreadsheets/d/1lVODoMORbDdVXwTG4RUZPWCSrx8uPx2bvwvXg5tuQYE/edit#gid=165427832"),
    ("Ore's Mega Dot OLLs", "https://docs.google.com/spreadsheets/d/17Ym3zUyYXX9NzFcPw5iCs-3fXAjfqCezK-Dsofv8A2Y/edit#gid=288801998"),
    ("Best 2x2 Algs", "https://docs.google.com/spreadsheets/d/1OFXakCV85Mp2zsQBXMxiMX9a506JeAcLnUXZr8FgXAY/edit#gid=1684065307"),
    ("Frederik's FH Algs", "https://crazymonkeyonspeed.github.io/"),
    ("Brian Sun 3x3 Algs", "https://www.youtube.com/@BrinaSun"),
    ("SpeedCubeDB", "https://speedcubedb.com/"),
    ("FTO TCP", "https://docs.google.com/document/d/1nkgNFdAu9v6p871urGrdkKD-QGmftYhjqZjD75z4Wqo/edit#heading=h.zf1ib2cgt533"),
    ("2H CMLL", "https://docs.google.com/spreadsheets/d/1uc5V3G-kRH3qV5b6Lq1yzTjnqSMS3D3dZcsCIOiJqcI/edit?gid=1409661599#gid=1409661599"),
    ("OH CMLL", "https://docs.google.com/spreadsheets/d/1WHnksK4yyv63sv3Es-PuKfnFRtZGafqp6rEuEfOx-cg/edit?gid=1409661599#gid=1409661599"),
    ("OH PLL", "https://docs.google.com/spreadsheets/d/e/2PACX-1vSKmPOKUtF0g1FUn7wEFtyZJFYegBXuYxlA4ZcUDYmoqC4xBQOXnsaKgAydGoohlI-c97HeRpMyec7d/pubhtml?gid=0&single=true"),
    ("Juju ZBLL", "https://docs.google.com/spreadsheets/d/1-uwmZHf4vwJxFgeB3-TiF8MQ0RFSS30d5CUK96PoIwk/edit?gid=0#gid=0"),
    ("Anthony and Jonas OH OLL", "https://docs.google.com/presentation/d/1cD9AdZ1GYT2Jxznlbtn1yNXOnAATZRgEW1I2qcyorok/edit?slide=id.p#slide=id.p"),
    ("ZBLS Tao Yu", "https://docs.google.com/spreadsheets/d/1s8Q2VM2c1eV3oLIxFfM4exvABO0AcCeMCUdMiYKnzC0/edit?gid=0#gid=0"),
    ("Daniel's PBL Document", "https://docs.google.com/spreadsheets/d/1VQNYNwdOLqqBkacHcfYtEBst22FOVhH9EAhTOYOZTgo/edit?gid=467747530#gid=467747530"),
    ("Elliot Full Floating", "https://docs.google.com/spreadsheets/d/1txRjB-fAWWSM-1e1w-eJbDp9xhu0ZDwLWQdswNZhcew/edit?gid=804375502#gid=804375502"),
    ("Zach Wegner's LBT Algs", "https://zwegner.github.io/cubing/fto/lbt-algs.html"),
    ("2-Look Last Triplet (Nautilus)", "https://docs.google.com/spreadsheets/d/1Dh4MEQzpCuTylqnkn09UxBEvEq9MNYMqMiWQgFsBuYE/edit?gid=418973378#gid=418973378"),
    ("FSLL Introduction and Algs", "https://docs.google.com/spreadsheets/d/1Z5tekV-6TZExwB9kylOtuQk7X0Zkt5llIfz11Ydkrk4/edit?gid=554409358#gid=554409358"),
];

const OTHER_LINKS: [(&str, &str); 4] = [
    (
        "Reinier's OLL Trainer",
        "https://reinierschippers.nl/oll_trainer-master/",
    ),
    ("Roman's OLL Trainer", "https://bestsiteever.ru/oll/"),
    ("Batch Solver", "https://trangium.github.io/BatchSolver/"),
    ("GitHub", "https://github.com/mihlefeld/Alg-Trainers"),
];

#[derive(Properties, PartialEq)]
pub struct CreditsDialogProps {
    pub on_close: Callback<()>,
}

#[function_component(CreditsDialog)]
pub fn credits_dialog(props: &CreditsDialogProps) -> Html {
    html! {
        <ModalDialog id="aboutDialog" on_close={props.on_close.clone()}>
            <h2>{"Credits and algorithm sources"}</h2>
            <div class="creditsGrid">
                <div class="colFlex">
                    <b>{"Algorithm sheets"}</b>
                    {for ALG_SHEET_LINKS.into_iter().map(|(label, href)| html! {
                        <a {href} target="_blank">{label}</a>
                    })}
                </div>
                <div class="colFlex">
                    <b>{"Other"}</b>
                    {for OTHER_LINKS.into_iter().map(|(label, href)| html! {
                        <a {href} target="_blank">{label}</a>
                    })}
                </div>
            </div>
        </ModalDialog>
    }
}
