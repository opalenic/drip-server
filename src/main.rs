use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler};

use ciborium::{de, ser};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum OutgoingMsg {
    TypeA(u16),
    TypeB(u32),
}

#[derive(Debug, Serialize, Deserialize)]
struct IncomingMsg(u8);


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


#[derive(Debug)]
struct DripNodeWs;

impl Actor for DripNodeWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DripNodeWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("{msg:?}");
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                let deserialized: OutgoingMsg = de::from_reader(&*bin).unwrap();
                println!("Deserialized this: {deserialized:?}, raw {bin:?}");
                let val = match deserialized {
                    OutgoingMsg::TypeA(v) => v as u8,
                    OutgoingMsg::TypeB(v) => v as u8
                };

                let mut serialized = Vec::new();
                ser::into_writer(&IncomingMsg(val), &mut serialized);

                ctx.binary(serialized)
            },
            _ => (),
        }
    }
}



async fn create_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(DripNodeWs, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .route("/ws", web::get().to(create_ws))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
