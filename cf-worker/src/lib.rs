use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    // Log the request method and URL
    let captured_path = req.path();
    Response::ok(&captured_path.clone())
}