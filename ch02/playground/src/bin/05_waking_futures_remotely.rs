use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

// Definición de la estructura MyFuture
struct MyFuture {
    state: Arc<Mutex<MyFutureState>>,
}

impl MyFuture {
    // Método para crear una nueva instancia de MyFuture
    fn new() -> (Self, Arc<Mutex<MyFutureState>>) {
        // Inicializar el estado con datos y waker vacíos
        let state = Arc::new(Mutex::new(MyFutureState {
            data: None,
            waker: None,
        }));
        (
            MyFuture {
                state: state.clone(),
            },
            state,
        )
    }
}

// Implementación del trait Future para MyFuture
impl Future for MyFuture {
    type Output = String;

    // Método poll que se llama repetidamente para verificar el estado del futuro
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Polling the future...");

        // Bloquear el estado para obtener acceso seguro
        let mut state = self.state.lock().unwrap();
        if state.data.is_some() {
            // Si los datos están disponibles, tomar los datos y devolver Poll::Ready
            let data = state.data.take().unwrap();
            Poll::Ready(String::from_utf8(data).unwrap())
        } else {
            // Si no hay datos disponibles, guardar el waker y devolver Poll::Pending
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// Definición de la estructura MyFutureState
struct MyFutureState {
    data: Option<Vec<u8>>,
    waker: Option<Waker>,
}

// Función main asincrónica, punto de entrada del programa
#[tokio::main]
async fn main() {
    // Crear una nueva instancia de MyFuture y obtener su estado
    let (my_future, state) = MyFuture::new();

    // Crear un canal mpsc para comunicación asincrónica
    let (tx, mut rx) = mpsc::channel::<()>(1);

    // Crear una tarea asincrónica que espera la resolución del futuro
    let task_handle = task::spawn(async { my_future.await });

    // Esperar 3 segundos antes de continuar
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("spawning trigger task");

    // Crear una tarea asincrónica que activa la resolución del futuro
    let trigger_task = task::spawn(async move {
        // Esperar a recibir un mensaje en el canal
        rx.recv().await;
        // Bloquear el estado para modificarlo
        let mut state = state.lock().unwrap();
        state.data = Some(b"Hello from the outside".to_vec());

        // Despertar al futuro si hay un waker registrado
        loop {
            if let Some(waker) = state.waker.take() {
                waker.wake();
                break;
            }
        }
    });

    // Enviar un mensaje en el canal para activar el futuro
    tx.send(()).await.unwrap();

    // Esperar a que la tarea se complete y obtener el resultado
    let outcome = task_handle.await.unwrap();
    println!("Task completed with outcome: {}", outcome);

    // Esperar a que la tarea de activación se complete
    trigger_task.await.unwrap();
}
