
fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {

    use tokio::runtime::Builder;
    use mockito::Matcher;
    use reqwest;

    #[test]
    fn test_networking() {

        let mut server = mockito::Server::new();
        let url = server.url();

        // Create a mock
        let mock = server.mock("GET", "/my-endpoint")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("param1".into(), "value1".into()),
            Matcher::UrlEncoded("param2".into(), "value2".into()),
        ]))
        .with_status(201)
        .with_body("world")
        .expect(5)
        .create();


        let runtime = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let mut handles = vec![];

        for _ in 0..5 {
            let url_clone = url.clone();
            handles.push(runtime.spawn(async move {
                let client = reqwest::Client::new();
                client.get(&format!("{}/my-endpoint?param1=value1&param2=value2", url_clone))
                      .send()
                      .await
                      .unwrap()
            }));
        }

        for handle in handles {
            runtime.block_on(handle).unwrap();
        }
        mock.assert();
    }
}

