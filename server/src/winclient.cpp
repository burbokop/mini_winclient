#include "winclient.h"

#include <QQmlInfo>
#include <QTcpSocket>

#define WARN_WITH_MSG(cond, msg) \
    if (!(cond)) { \
        qmlWarning(giveMeErr) << msg; \
        return std::nullopt; \
    }

namespace {

template<typename T>
std::optional<T> read(QIODevice *dev)
{
    if (dev->bytesAvailable() >= sizeof(T)) {
        const auto bytes = dev->read(sizeof(T));
        assert(bytes.size() == sizeof(T));
        const auto ptr = reinterpret_cast<const T *>(bytes.constData());
        return *ptr;
    } else {
        return std::nullopt;
    }
}

template<typename T>
void write(QIODevice *dev, const T &v)
{
    const auto ptr = reinterpret_cast<const char *>(&v);
    assert(dev->write(ptr, sizeof(T)) == sizeof(T));
}

std::optional<QImage::Format> parseFormat(std::uint8_t fmt)
{
    switch (fmt) {
    case 0:
        return QImage::Format_Grayscale8;
    case 1:
        return QImage::Format_ARGB32;
    default:
        return std::nullopt;
    }
}

} // namespace

std::optional<WinClient::Frame> WinClient::parseFrame(QByteArray &&arr,
                                                      std::uint8_t cid,
                                                      QObject *giveMeErr)
{
    QBuffer buf(&arr);
    buf.open(QIODevice::ReadOnly);

    const auto protoVersion = read<std::uint8_t>(&buf);
    assert(protoVersion);
    const auto clientId = read<std::uint8_t>(&buf);
    assert(clientId);
    WARN_WITH_MSG(*clientId == cid, "Client id does not match");
    const auto packageType = read<std::uint8_t>(&buf);
    assert(packageType);
    WARN_WITH_MSG(*packageType == 0, "Server suports only package type 0 (present)");
    const auto format = read<std::uint8_t>(&buf);
    assert(format);
    const auto pixelSize = read<std::uint8_t>(&buf);
    assert(pixelSize);
    const auto w = read<std::uint16_t>(&buf);
    assert(w);
    const auto h = read<std::uint16_t>(&buf);
    assert(h);
    QByteArray pixels = buf.readAll();

    WARN_WITH_MSG(pixels.size() == *w * *h * *pixelSize,
                  "Parsing request failed: invalid pix array size");
    WARN_WITH_MSG(*protoVersion == 0, "Server supports only protocol version 0");

    const auto fmt = parseFormat(*format);
    WARN_WITH_MSG(fmt, "Unknown pix format: " << *format);
    const QImage image(reinterpret_cast<const uchar *>(pixels.constData()), *w, *h, *fmt);

    WARN_WITH_MSG(!image.isNull(), "Invalid image");
    WARN_WITH_MSG(image.depth() == (*pixelSize * 8),
                  "Image depth not match with received pixel size");

    return Frame{.protoVersion = *protoVersion,
                 .format = *protoVersion,
                 .pixelSize = *pixelSize,
                 .w = *w,
                 .h = *h,
                 .image = image.copy()};
}

WinClient::WinClient(QTcpSocket *socket, uint8_t clientId, QObject *parent)
    : QObject(parent)
    , m_socket(socket)
    , m_id(clientId)
{
    write(m_socket,
          std::uint32_t(sizeof(std::uint8_t)   // package type
                        + sizeof(std::uint8_t) // client id
                        ));
    write(m_socket, std::uint8_t(0));
    write(m_socket, m_id);
    if (!m_socket->waitForBytesWritten()) {
        qmlWarning(this) << "Can not write init package: " << m_socket->errorString();
        QMetaObject::invokeMethod(this, [this]() { emit dead(QPrivateSignal()); });
        return;
    }

    connect(m_socket, &QTcpSocket::readyRead, this, [this]() {
        if (!m_currentPackageSize) {
            if (const auto packageSize = read<std::uint32_t>(m_socket)) {
                m_currentPackageSize = *packageSize;
            }
        }

        if (m_currentPackageSize) {
            if (m_socket->bytesAvailable() >= *m_currentPackageSize) {
                if (const auto frame = parseFrame(m_socket->read(*m_currentPackageSize),
                                                  m_id,
                                                  this)) {
                    m_frame = *frame;
                    if (width() != m_prevW) {
                        emit widthChanged();
                        m_prevW = width();
                    }
                    if (height() != m_prevH) {
                        emit heightChanged();
                        m_prevH = height();
                    }
                    emit imageChanged();
                    emit titleChanged();
                }
                m_currentPackageSize = std::nullopt;
            }
        }
    });

    connect(m_socket, &QTcpSocket::disconnected, this, [this]() {
        m_socket->deleteLater();
        m_socket = nullptr;
        emit dead(QPrivateSignal());
    });
}

void WinClient::sendCloseEvent()
{
    qmlWarning(this) << "TODO: WinClient::sendCloseEvent";
}

QString WinClient::title() const
{
    return "Client " + QString::number(m_id);
}
