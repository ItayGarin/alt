//
// Ex: i3-msg -t subscribe -m '[ "window" ]'
//

std::net::TcpListener;

struct i3Focus {
    focused: String,
    socket: TcpListener
}
