
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QGuiApplication>
#include <QQmlApplicationEngine>

struct Args
{
    quint16 port;
};

namespace {
std::optional<Args> parseArgs(const QStringList &args)
{
    static const auto portOption = QCommandLineOption{{"p", "port"}, "Port of server", "PORT", 0};
    static const auto helpOption = QCommandLineOption{{"h", "help"}};

    QCommandLineParser parser;
    parser.addOption(portOption);
    parser.addOption(helpOption);
    parser.parse(args);

    if (parser.isSet(helpOption)) {
        qDebug() << parser.helpText().toUtf8().constData();
        return std::nullopt;
    }

    return Args{.port = parser.value(portOption).toUShort(nullptr)};
}
} // namespace

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    if (const auto args = parseArgs(app.arguments())) {
        QQmlApplicationEngine engine;
        engine.setInitialProperties({{"port", args->port}});
        const QUrl url(u"qrc:/org/burbokop/mini_winserver/Main.qml"_qs);
        QObject::connect(
            &engine,
            &QQmlApplicationEngine::objectCreationFailed,
            &app,
            []() { QCoreApplication::exit(-2); },
            Qt::QueuedConnection);
        engine.load(url);
        return app.exec();
    }
    return 0;
}
