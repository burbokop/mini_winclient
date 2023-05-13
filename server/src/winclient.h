#pragma once

#include <QImage>
#include <QObject>
#include <QtQmlIntegration>

class QTcpSocket;

class WinClient : public QObject
{
    Q_OBJECT
    QML_UNCREATABLE("WinServer creates clients");
    struct Frame
    {
        std::uint8_t protoVersion;
        std::uint8_t format;
        std::uint8_t pixelSize;
        std::uint16_t w;
        std::uint16_t h;
        QImage image;
    };

    Q_PROPERTY(QImage image READ image NOTIFY imageChanged)
    Q_PROPERTY(int width READ width NOTIFY widthChanged)
    Q_PROPERTY(int height READ height NOTIFY heightChanged)
    Q_PROPERTY(QString title READ title NOTIFY titleChanged)

public:
    WinClient(QTcpSocket *socket, std::uint8_t clientId, QObject *parent = nullptr);

    void sendCloseEvent();

    const QImage &image() const
    {
        static const QImage null;
        return m_frame ? m_frame->image : null;
    }
    int width() const { return m_frame ? m_frame->w : 0; };
    int height() const { return m_frame ? m_frame->h : 0; };

    QString title() const;
    std::uint8_t id() const { return m_id; };

signals:
    void dead(QPrivateSignal);

    void imageChanged();
    void widthChanged();
    void heightChanged();
    void titleChanged();

private:
    static std::optional<Frame> parseFrame(QByteArray &&arr, std::uint8_t cid, QObject *giveMeErr);

    std::optional<std::uint32_t> m_currentPackageSize;
    std::optional<Frame> m_frame;
    QTcpSocket *m_socket = nullptr;
    std::uint8_t m_id = 0;

    int m_prevW = 0;
    int m_prevH = 0;
};
